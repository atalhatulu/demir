# Demir Language Architecture & Roadmap (V0.1)

Bu doküman projenin genel vizyonunu, alınan temel mimari kararları ve yol haritasını özetler.

## 1. Bootstrapping ve Rust Kararı
Compiler'ın V0.1 implementasyon dili **Rust** olarak kesinleştirilmiştir.
- Rust, Demir'in kendisi değildir; sadece derleyiciyi (compiler) geliştirmek için kullanılan bootstrap dilidir.
- Mimari: `Rust -> Demir Compiler -> Demir Source -> Native Executable`
- **Self-hosting (bootstrapping)** uzun vadeli bir hedeftir (V2+). Şu an için compiler'ı Demir'e taşıma hedefimiz yoktur.

## 2. Kaynak Dil (Source Language)
Projenin adı **Demir**, uzantısı ise **.dmr** olarak belirlenmiştir.
- Dil ve compiler isimleri kod içerisinde hard-code edilmemeli, ileride rahat değiştirilebilecek bir constant yapısına bağlanmalıdır.

## 3. JIT (Just-in-Time)
Hızlı geliştirme ve test döngüleri için JIT execution birinci sınıf vatandaş olarak kalacaktır.

## 4. Cranelift (AOT + JIT)
LLVM'in ağır derleme sürelerinden kaçınmak ve JIT entegrasyonunu sağlamak için **Cranelift** (WebAssembly tabanlı JIT engine) kullanılacaktır.

## 5. Yol Haritası (Kısa Vadeli)
- [x] Temel AST -> Cranelift bağlaması (Bağlandı)
- [x] JIT Execution ile script çalıştırma
- [x] Değişken (let/var), Atama, Aritmetik, Block ve If desteği
- [x] Borrow Checker mimarisi kurulumu (ownership/move takibi + stack-slot tabanlı `&`/`&mut`/`*` pointer codegen)
- [x] Design-by-contract (`requires`/`ensures`) runtime kontrolü (ASSERT FAILED + exit 1)
- [x] Cranelift Object generation (AOT) ve native binary
- [ ] Gelişmiş Type System (Structs, Trait/Interface benzeri)
- [ ] AI-first sentaks kuralları (Agent, Intent) — V0.3+ araştırma
- [ ] LLM Native Bindings (İleri aşama)
- [ ] Hafıza modeli (Memory allocation: arena/region-based — önerilen)
