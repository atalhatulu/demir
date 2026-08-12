use std::collections::{HashSet, HashMap};
use crate::mir::*;

pub struct CfgGraph {
    pub num_blocks: usize,
    pub preds: Vec<Vec<BasicBlockId>>,
    pub succs: Vec<Vec<BasicBlockId>>,
}

impl CfgGraph {
    pub fn build(mir_func: &MirFunction) -> Self {
        let num_blocks = mir_func.blocks.len();
        let mut preds = vec![Vec::new(); num_blocks];
        let mut succs = vec![Vec::new(); num_blocks];

        for (i, block) in mir_func.blocks.iter().enumerate() {
            let curr = BasicBlockId(i);
            if let Some(term) = &block.terminator {
                match term {
                    Terminator::Goto { target } => {
                        succs[i].push(*target);
                        preds[target.0].push(curr);
                    }
                    Terminator::If { then_target, else_target, .. } => {
                        succs[i].push(*then_target);
                        preds[then_target.0].push(curr);
                        
                        succs[i].push(*else_target);
                        preds[else_target.0].push(curr);
                    }
                    Terminator::Call { target, .. } => {
                        succs[i].push(*target);
                        preds[target.0].push(curr);
                    }
                    Terminator::Return(_) => {}
                }
            }
        }

        Self { num_blocks, preds, succs }
    }
}

pub struct SsaAnalyzer {
    pub cfg: CfgGraph,
    pub doms: Vec<HashSet<BasicBlockId>>,          
    pub idom: Vec<Option<BasicBlockId>>,           
    pub dom_tree: Vec<Vec<BasicBlockId>>,          
    pub dom_frontier: Vec<HashSet<BasicBlockId>>,  
}

impl SsaAnalyzer {
    pub fn new(mir_func: &MirFunction) -> Self {
        let cfg = CfgGraph::build(mir_func);
        let num_blocks = cfg.num_blocks;

        let mut analyzer = Self {
            cfg,
            doms: vec![HashSet::new(); num_blocks],
            idom: vec![None; num_blocks],
            dom_tree: vec![Vec::new(); num_blocks],
            dom_frontier: vec![HashSet::new(); num_blocks],
        };

        if num_blocks > 0 {
            analyzer.compute_dominators();
            analyzer.compute_idom();
            analyzer.compute_dominance_frontier();
        }

        analyzer
    }

    fn compute_dominators(&mut self) {
        let num_blocks = self.cfg.num_blocks;
        let mut all_nodes = HashSet::new();
        for i in 0..num_blocks {
            all_nodes.insert(BasicBlockId(i));
        }

        self.doms[0].insert(BasicBlockId(0));

        for i in 1..num_blocks {
            self.doms[i] = all_nodes.clone();
        }

        let mut changed = true;
        while changed {
            changed = false;

            for i in 1..num_blocks {
                let curr = BasicBlockId(i);
                let preds = &self.cfg.preds[i];
                
                if preds.is_empty() {
                    continue; 
                }

                let mut new_dom = self.doms[preds[0].0].clone();
                for p in preds.iter().skip(1) {
                    let p_doms = &self.doms[p.0];
                    new_dom.retain(|x| p_doms.contains(x));
                }

                new_dom.insert(curr);

                if new_dom != self.doms[i] {
                    self.doms[i] = new_dom;
                    changed = true;
                }
            }
        }
    }

    fn compute_idom(&mut self) {
        for i in 1..self.cfg.num_blocks {
            let curr = BasicBlockId(i);
            let mut strict_doms = self.doms[i].clone();
            strict_doms.remove(&curr); 
            
            let mut idom = None;
            for d in strict_doms.iter() {
                let mut is_idom = true;
                for other in strict_doms.iter() {
                    if d != other && !self.doms[d.0].contains(other) {
                        is_idom = false;
                        break;
                    }
                }
                if is_idom {
                    idom = Some(*d);
                    break;
                }
            }

            self.idom[i] = idom;
            if let Some(id) = idom {
                self.dom_tree[id.0].push(curr);
            }
        }
    }

    fn compute_dominance_frontier(&mut self) {
        for b in 0..self.cfg.num_blocks {
            let preds = &self.cfg.preds[b];
            if preds.len() >= 2 { 
                for p in preds {
                    let mut runner = p.0;
                    while Some(BasicBlockId(runner)) != self.idom[b] {
                        self.dom_frontier[runner].insert(BasicBlockId(b));
                        
                        if let Some(idm) = self.idom[runner] {
                            runner = idm.0;
                        } else {
                            break; 
                        }
                    }
                }
            }
        }
    }
}

pub fn place_phi_nodes(mir_func: &mut MirFunction, analyzer: &SsaAnalyzer) {
    let mut defs: HashMap<LocalId, HashSet<BasicBlockId>> = HashMap::new();
    
    // 1. Değişkenlerin tanımlandığı blokları bul (Assignments)
    for (b_idx, block) in mir_func.blocks.iter().enumerate() {
        let b_id = BasicBlockId(b_idx);
        for stmt in &block.statements {
            if let Statement::Assign(dest, _) = stmt {
                defs.entry(*dest).or_insert_with(HashSet::new).insert(b_id);
            }
        }
        if let Some(Terminator::Call { destination, .. }) = &block.terminator {
            defs.entry(*destination).or_insert_with(HashSet::new).insert(b_id);
        }
    }

    // 2. Cytron'un Phi Yerleşim (Placement) Algoritması
    for (local, def_blocks) in defs.iter() {
        let mut worklist: Vec<BasicBlockId> = def_blocks.iter().cloned().collect();
        let mut has_phi: HashSet<BasicBlockId> = HashSet::new(); 

        while let Some(b) = worklist.pop() {
            for df_node in &analyzer.dom_frontier[b.0] {
                if !has_phi.contains(df_node) {
                    
                    let mut operands = Vec::new();
                    for p in &analyzer.cfg.preds[df_node.0] {
                        operands.push((*p, *local)); // (Predecessor block, Value ID)
                    }
                    
                    mir_func.blocks[df_node.0].phi_nodes.push(PhiNode {
                        dest: *local,
                        operands,
                    });

                    has_phi.insert(*df_node);
                    
                    // Phi eklenen blok da artık bu değişken için bir "def" noktası sayılır
                    // Eğer worklist'te veya ana def listesinde yoksa ekle.
                    if !def_blocks.contains(df_node) {
                        worklist.push(*df_node);
                    }
                }
            }
        }
    }
}

pub fn build_ssa(mir_func: &mut MirFunction) {
    let analyzer = SsaAnalyzer::new(mir_func);
    if analyzer.cfg.num_blocks == 0 { return; }
    
    place_phi_nodes(mir_func, &analyzer);
    rename_variables(mir_func, &analyzer);
    validate_ssa(mir_func, &analyzer).expect("SSA Validation Failed");
}

fn rename_variables(mir_func: &mut MirFunction, analyzer: &SsaAnalyzer) {
    let mut stacks: HashMap<LocalId, Vec<LocalId>> = HashMap::new();
    let mut next_id = 20000;
    
    let mut orig_locals = HashMap::new();
    for loc in &mir_func.locals {
        stacks.insert(loc.id, vec![loc.id]);
        orig_locals.insert(loc.id, loc.clone());
    }

    let mut new_decls = Vec::new();
    
    rename_block(
        BasicBlockId(0),
        mir_func,
        analyzer,
        &mut stacks,
        &mut next_id,
        &orig_locals,
        &mut new_decls
    );

    mir_func.locals.extend(new_decls);
}

fn rename_operand(op: &mut Operand, stacks: &HashMap<LocalId, Vec<LocalId>>) {
    match op {
        Operand::Copy(id) | Operand::Move(id) => {
            if let Some(stack) = stacks.get(id) {
                if let Some(top) = stack.last() {
                    *id = *top;
                }
            }
        }
        _ => {}
    }
}

fn rename_block(
    b: BasicBlockId,
    mir_func: &mut MirFunction,
    analyzer: &SsaAnalyzer,
    stacks: &mut HashMap<LocalId, Vec<LocalId>>,
    next_id: &mut usize,
    orig_locals: &HashMap<LocalId, LocalDecl>,
    new_decls: &mut Vec<LocalDecl>,
) {
    let mut pushed_versions = Vec::new();

    for i in 0..mir_func.blocks[b.0].phi_nodes.len() {
        let orig_dest = mir_func.blocks[b.0].phi_nodes[i].dest;
        let new_id = LocalId(*next_id);
        *next_id += 1;
        
        stacks.entry(orig_dest).or_insert(vec![]).push(new_id);
        pushed_versions.push(orig_dest);
        mir_func.blocks[b.0].phi_nodes[i].dest = new_id;

        if let Some(decl) = orig_locals.get(&orig_dest) {
            new_decls.push(LocalDecl { id: new_id, ty: decl.ty.clone(), ownership: decl.ownership.clone(), is_mut: decl.is_mut });
        }
    }

    for i in 0..mir_func.blocks[b.0].statements.len() {
        let mut stmt = mir_func.blocks[b.0].statements[i].clone();
        match &mut stmt {
            Statement::Assign(dest, rval) => {
                match rval {
                    Rvalue::Use(op) => {
                        rename_operand(op, stacks);
                    }
                    Rvalue::BinaryOp(_, left, right) => {
                        rename_operand(left, stacks);
                        rename_operand(right, stacks);
                    }
                    Rvalue::StructAlloc(_) => {}
                    Rvalue::FieldLoad(ptr, _) | Rvalue::AddressOf(ptr) | Rvalue::MutAddressOf(ptr) | Rvalue::Dereference(ptr) => {
                        let mut op = Operand::Copy(*ptr);
                        rename_operand(&mut op, stacks);
                        if let Operand::Copy(new_ptr) = op {
                            *ptr = new_ptr;
                        }
                    }
                }
                let orig_dest = *dest;
                let new_id = LocalId(*next_id);
                *next_id += 1;
                
                stacks.entry(orig_dest).or_insert(vec![]).push(new_id);
                pushed_versions.push(orig_dest);
                *dest = new_id;
                
                if let Some(decl) = orig_locals.get(&orig_dest) {
                    new_decls.push(LocalDecl { id: new_id, ty: decl.ty.clone(), ownership: decl.ownership.clone(), is_mut: decl.is_mut });
                }
            }
            Statement::Store(ptr, _, val) => {
                let mut op = Operand::Copy(*ptr);
                rename_operand(&mut op, stacks);
                if let Operand::Copy(new_ptr) = op {
                    *ptr = new_ptr;
                }
                rename_operand(val, stacks);
            }
            Statement::Assert(cond, _) => {
                rename_operand(cond, stacks);
            }
        }
        mir_func.blocks[b.0].statements[i] = stmt;
    }

    if let Some(mut term) = mir_func.blocks[b.0].terminator.clone() {
        match &mut term {
            Terminator::If { cond, .. } => rename_operand(cond, stacks),
            Terminator::Return(Some(op)) => rename_operand(op, stacks),
            Terminator::Call { args, destination, .. } => {
                for arg in args {
                    rename_operand(arg, stacks);
                }
                let orig_dest = *destination;
                let new_id = LocalId(*next_id);
                *next_id += 1;
                
                stacks.entry(orig_dest).or_insert(vec![]).push(new_id);
                pushed_versions.push(orig_dest);
                *destination = new_id;
                
                if let Some(decl) = orig_locals.get(&orig_dest) {
                    new_decls.push(LocalDecl { id: new_id, ty: decl.ty.clone(), ownership: decl.ownership.clone(), is_mut: decl.is_mut });
                }
            },
            _ => {}
        }
        mir_func.blocks[b.0].terminator = Some(term);
    }

    let succs = analyzer.cfg.succs[b.0].clone();
    for succ in succs {
        for phi in &mut mir_func.blocks[succ.0].phi_nodes {
            for op in &mut phi.operands {
                if op.0 == b {
                    if let Some(stack) = stacks.get(&op.1) {
                        if let Some(top) = stack.last() {
                            op.1 = *top;
                        }
                    }
                }
            }
        }
    }

    let children = analyzer.dom_tree[b.0].clone();
    for child in children {
        rename_block(child, mir_func, analyzer, stacks, next_id, orig_locals, new_decls);
    }

    for orig_dest in pushed_versions.iter().rev() {
        if let Some(stack) = stacks.get_mut(orig_dest) {
            stack.pop();
        }
    }
}

pub fn validate_ssa(mir_func: &MirFunction, analyzer: &SsaAnalyzer) -> Result<(), String> {
    let mut defs = HashMap::new();
    let mut uses = Vec::new();

    for (b_idx, block) in mir_func.blocks.iter().enumerate() {
        let b = BasicBlockId(b_idx);
        
        for phi in &block.phi_nodes {
            if defs.contains_key(&phi.dest) {
                return Err(format!("SSA Validation Error: Local {} is defined multiple times (phi)", phi.dest.0));
            }
            defs.insert(phi.dest, b);
            
            let preds = &analyzer.cfg.preds[b.0];
            if phi.operands.len() != preds.len() {
                return Err(format!("SSA Validation Error: Phi node {} has {} operands, but block {} has {} predecessors", phi.dest.0, phi.operands.len(), b.0, preds.len()));
            }
            
            for (pred, arg) in &phi.operands {
                if !preds.contains(pred) {
                    return Err(format!("SSA Validation Error: Phi operand block {} is not a predecessor of {}", pred.0, b.0));
                }
                uses.push((*arg, *pred));
            }
        }
        
        for stmt in &block.statements {
            match stmt {
                Statement::Assign(dest, rval) => {
                    if defs.contains_key(dest) {
                        return Err(format!("SSA Validation Error: Local {} is defined multiple times (stmt)", dest.0));
                    }
                    defs.insert(*dest, b);
                    
                    match rval {
                        Rvalue::Use(op) => {
                            extract_use(op, b, &mut uses);
                        }
                        Rvalue::BinaryOp(_, left, right) => {
                            extract_use(left, b, &mut uses);
                            extract_use(right, b, &mut uses);
                        }
                        Rvalue::StructAlloc(_) => {}
                        Rvalue::FieldLoad(ptr, _) | Rvalue::AddressOf(ptr) | Rvalue::MutAddressOf(ptr) | Rvalue::Dereference(ptr) => {
                            let op = Operand::Copy(*ptr);
                            extract_use(&op, b, &mut uses);
                        }
                    }
                }
                Statement::Store(ptr, _, val) => {
                    let op = Operand::Copy(*ptr);
                    extract_use(&op, b, &mut uses);
                    extract_use(val, b, &mut uses);
                }
                Statement::Assert(cond, _) => {
                    extract_use(cond, b, &mut uses);
                }
            }
        }
        
        if let Some(term) = &block.terminator {
            match term {
                Terminator::If { cond, .. } => extract_use(cond, b, &mut uses),
                Terminator::Return(Some(op)) => extract_use(op, b, &mut uses),
                Terminator::Call { args, destination, .. } => {
                    for arg in args {
                        extract_use(arg, b, &mut uses);
                    }
                    if defs.contains_key(destination) {
                        return Err(format!("SSA Validation Error: Local {} is defined multiple times (call)", destination.0));
                    }
                    defs.insert(*destination, b);
                }
                _ => {}
            }
        }
    }

    for (used_val, use_block) in uses {
        if used_val.0 < 20000 {
            continue;
        }
        if let Some(def_block) = defs.get(&used_val) {
            if def_block != &use_block && !analyzer.doms[use_block.0].contains(def_block) {
                return Err(format!("SSA Validation Error: Use of {} in bb{} is not dominated by its definition in bb{}", used_val.0, use_block.0, def_block.0));
            }
        } else {
            return Err(format!("SSA Validation Error: Use of undefined SSA value {} in bb{}", used_val.0, use_block.0));
        }
    }

    Ok(())
}

fn extract_use(op: &Operand, block: BasicBlockId, uses: &mut Vec<(LocalId, BasicBlockId)>) {
    match op {
        Operand::Copy(id) | Operand::Move(id) => uses.push((*id, block)),
        _ => {}
    }
}
