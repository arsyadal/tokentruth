# PRD: TokenTruth
**Independent verification untuk klaim penghematan token AI coding agent**

| | |
|---|---|
| **Versi dokumen** | 0.1 (Draft) |
| **Tanggal** | 20 Juli 2026 |
| **Status** | Pre-development |
| **Pemilik** | (isi nama kamu) |

---

## 1. Ringkasan Eksekutif

TokenTruth adalah CLI open source yang membaca log sesi asli dari AI coding agent (Claude Code, dengan ekspansi ke Codex CLI dan Cursor) dan menghitung **penghematan token yang sebenarnya terjadi** — bukan estimasi, bukan angka marketing dari tool pihak ketiga (Caveman, RTK, Ponytail, Headroom), tapi angka yang direkonstruksi dari transcript mentah di mesin pengguna sendiri.

**Elevator pitch:** *"Kamu install lima tool yang katanya hemat token. Tapi berapa penghematan ASLI di sesi kamu, dihitung dari log yang benar-benar terjadi?"*

---

## 2. Latar Belakang & Masalah

### 2.1 Konteks pasar

- Token spend agentic kini jadi pos biaya utama tim engineering; sesi agentic tunggal bisa memakan lebih banyak token daripada seminggu pemakaian chat biasa.
- Rasio input:output pada sesi agentic sekitar 20-25:1 — artinya harga **input** token yang sebenarnya menentukan tagihan, bukan output.
- Ekosistem tool "penghemat token" (Caveman, RTK, Ponytail, Headroom, dll) tumbuh sangat cepat (puluhan-ratusan ribu stars dalam hitungan minggu), tapi mayoritas menyerang sisi **output** token, bukan input — padahal input adalah porsi terbesar biaya.

### 2.2 Masalah spesifik yang memicu proyek ini

Benchmark independen JetBrains terhadap skill "Caveman" menemukan gap besar antara klaim dan realita:

| Klaim marketing | Hasil benchmark independen (JetBrains, SkillsBench) |
|---|---|
| 65% token saved | 8.5% (output-only, ceiling case, bukan kasus umum) |

Bahkan repository Caveman sendiri kini mencantumkan "honest number warning": skill hanya mengecilkan output token, input & reasoning token tidak tersentuh, dan skill itu sendiri menambah ~1-1.5k token input per turn — sehingga pada workload yang sudah ringkas, hasilnya bisa **net-negatif**.

Tidak ada satupun tool di ekosistem ini yang:

1. Independen dari vendor tool kompresi (semua self-reported).
2. Mengukur dari log transcript mentah, bukan sampling/estimasi.
3. Memisahkan kategori token (input / output / reasoning / tool-call / cache).
4. Membandingkan lintas beberapa tool kompresi sekaligus di sesi nyata pengguna.

### 2.3 Peluang

Ruang "kompresi token" sudah padat dan didominasi pemain besar → **tidak** kompetitif untuk pendatang baru. Ruang "verifikasi klaim kompresi token" masih **kosong** dan punya sinyal permintaan jelas (artikel JetBrains, "honest warning" di README Caveman, keluhan developer soal token cost yang tidak transparan). Posisi TokenTruth: **penengah netral**, bukan pesaing tool kompresi — berpotensi direkomendasikan oleh tool-tool tersebut sendiri untuk validasi.

---

## 3. Tujuan Produk

### 3.1 Goals (MVP)

- G1: Developer bisa menjalankan satu perintah dan mendapat breakdown token per kategori dari sesi Claude Code mereka, dihitung dari transcript mentah.
- G2: Developer bisa membandingkan sesi "sebelum" vs "sesudah" mengaktifkan tool kompresi tertentu, dengan angka yang bisa dipertanggungjawabkan (bukan self-reported oleh tool tsb).
- G3: Developer bisa mengonversi token usage menjadi estimasi biaya riil (USD) lintas beberapa model/provider.
- G4: Menjadi rujukan netral yang cukup dipercaya sehingga disebut di komunitas (HN, Reddit, README tool lain) sebagai alat verifikasi independen.

### 3.2 Non-goals (eksplisit di luar scope MVP)

- Membuat tool kompresi sendiri (menimbulkan conflict of interest terhadap fungsi audit).
- Mendukung semua AI coding agent sekaligus di rilis pertama (fokus Claude Code dulu).
- Dashboard web / SaaS di fase awal (budaya audiens ini adalah CLI-first).
- Real-time monitoring/TUI ala "htop for agents" (sudah ada pemain di ceruk ini; TokenTruth adalah alat **audit post-hoc**, bukan monitor live).
- Audit billing enterprise / rekonsiliasi invoice API (ini domain produk lain seperti TokenAudit/Vaudit — beda target pasar, jangan tumpang tindih).

---

## 4. Target Pengguna

| Persona | Kebutuhan | Contoh trigger |
|---|---|---|
| **Individual power user** | Ingin tahu apakah skill/plugin token-saving yang ia install benar-benar berguna | "Saya install Caveman minggu lalu, worth it nggak sih?" |
| **Tech lead / staff eng** | Perlu data objektif sebelum merekomendasikan tool ke tim | "Sebelum saya standardisasi RTK ke seluruh tim, saya mau bukti dulu" |
| **Content creator / blogger teknis** | Butuh data untuk artikel perbandingan tool | Sumber dari riset kita sendiri: banyak blog post "X vs Y" yang datanya tidak diverifikasi |
| **Maintainer tool kompresi** | Ingin bukti pihak ketiga untuk klaim mereka sendiri | Potensi rekomendasi balik dari Caveman/RTK/Ponytail ke TokenTruth |

---

## 5. Lanskap Kompetitif

| Nama | Kategori | Perbedaan dengan TokenTruth |
|---|---|---|
| RTK, Caveman, Ponytail, Headroom | Tool **kompresi** token | Mereka melakukan optimasi; TokenTruth **mengukur** apakah optimasi itu bekerja. Bukan pesaing — bisa jadi objek audit. |
| "htop for AI agents" (TUI monitor) | **Monitoring real-time** sesi berjalan | TokenTruth adalah audit **post-hoc** dari transcript, bukan live monitor. Beda use case: retrospektif vs observasi langsung. |
| TokenAudit (tokenaudit.is / Vaudit) | Kalkulator biaya & audit billing **enterprise**, closed-source SaaS | Fokus TokenAudit: model mana paling murah + rekonsiliasi invoice API. TokenTruth: **verifikasi klaim tool kompresi** dari log sesi individual, open source. Target pasar berbeda (enterprise billing vs individual developer tooling). |
| tiktoken, Anthropic tokenizer | Penghitung token offline | Tidak punya integrasi IDE/CLI yang berarti, tidak ada cost modeling, tidak lintas provider. TokenTruth membangun di atas kebutuhan ini dengan konteks sesi penuh. |

**Catatan penting:** nama "TokenAudit" sudah dipakai (tokenaudit.is, terafiliasi Vaudit, sudah diberitakan Yahoo Finance). TokenTruth tidak boleh memakai nama itu untuk menghindari kebingungan pasar dan risiko trademark.

---

## 6. Spesifikasi Teknis

### 6.1 Sumber data

Claude Code menyimpan transcript lengkap tiap sesi sebagai file JSONL (satu objek JSON per baris, append-only) di:

```
~/.claude/projects/<encoded-project-path>/<session-uuid>.jsonl
```

(Windows: `%USERPROFILE%\.claude\projects\...`)

Karakteristik penting:

- Setiap baris adalah event bertipe: user turn, assistant response (dengan content blocks: text/tool_use/thinking), tool result, system event, summary.
- Setiap turn assistant membawa **token usage per turn** (termasuk breakdown cache read/write bila ada).
- Event terhubung lewat `parentUuid` membentuk chain percakapan.
- **Risiko:** format ini internal Claude Code dan bisa berubah antar versi tanpa notice resmi (dikonfirmasi dokumentasi resmi: "entry format is internal... scripts that parse these files directly can break on any release"). Ini punya implikasi langsung ke maintenance burden — lihat §9 Risiko.
- Retensi default: Claude Code dapat menghapus transcript lama secara otomatis (default ~30 hari) kecuali dikonfigurasi lain di `settings.json`. TokenTruth perlu mendeteksi & memperingatkan pengguna soal ini di first-run.

### 6.2 Arsitektur MVP

```
┌─────────────────┐     ┌──────────────┐     ┌───────────────┐     ┌────────────┐
│ ~/.claude/       │ --> │ JSONL Parser │ --> │ Token          │ --> │ CLI Output │
│ projects/*.jsonl │     │ (serde)      │     │ Categorizer +  │     │ (table /   │
└─────────────────┘     └──────────────┘     │ Cost Calculator│     │  JSON)     │
                                              └───────────────┘     └────────────┘
```

- **Bahasa:** Rust — selaras ekspektasi ekosistem (single binary, startup cepat, `cargo install tokentruth`). Gunakan `serde`/`serde_json` untuk parsing JSONL streaming (bukan load penuh ke memory — file bisa besar, salah satu sumber melaporkan 379MB akumulasi log).
- **Prinsip desain (meniru pola RTK yang sudah terbukti diterima komunitas):**
  - Startup < 10ms untuk operasi baca sederhana.
  - Read-only, tidak pernah menulis/memodifikasi file log pengguna.
  - Tidak ada network call di MVP (privacy-first, semua proses lokal) — jadi nilai jual tambahan: *"token data kamu tidak pernah meninggalkan mesin."*
  - Zero config untuk kasus dasar; opsional config file untuk kasus lanjutan.

### 6.3 Model data internal (skema kerja, disederhanakan)

```rust
struct SessionRecord {
    session_id: String,
    project_path: String,
    started_at: DateTime,
    ended_at: DateTime,
    turns: Vec<TurnUsage>,
}

struct TurnUsage {
    turn_id: String,
    role: Role, // User | Assistant | ToolResult | System
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_write_tokens: u64,
    reasoning_tokens: Option<u64>,
    model: String,           // e.g. "claude-sonnet-5"
    tool_name: Option<String>, // jika turn ini adalah tool call
}
```

### 6.4 Perhitungan biaya

Tabel pricing per model (Anthropic, OpenAI, Google, dll) disimpan sebagai data statis yang di-bundle, dengan mekanisme update manual per rilis di awal (bukan fetch otomatis — hindari dependency network di MVP). Roadmap fase 2: opsi update pricing via file config yang bisa di-refresh manual oleh user (`tokentruth pricing update --file pricing.json`), tetap tanpa call otomatis ke luar tanpa consent.

### 6.5 Command spec (draft)

```bash
# Analisis satu sesi terakhir di project saat ini
tokentruth analyze

# Analisis sesi spesifik
tokentruth analyze --session <uuid>

# Breakdown kategori token
tokentruth analyze --breakdown

# Bandingkan dua sesi (mis. sebelum/sesudah aktifkan RTK)
tokentruth compare --before <session-a> --after <session-b>

# Estimasi biaya lintas model untuk sesi yang sama
tokentruth cost --session <uuid> --models sonnet-5,gpt-5,gemini-3

# Export data mentah (untuk artikel/laporan)
tokentruth export --session <uuid> --format json|csv
```

---

## 7. Ruang Lingkup Rilis

### 7.1 MVP (v0.1) — target 2-3 minggu, full-time

- [ ] Parser JSONL Claude Code (single project, single session)
- [ ] Breakdown token: input / output / cache read / cache write
- [ ] Perhitungan biaya untuk model-model utama saat rilis (Sonnet, Opus, Haiku; GPT lini terbaru; Gemini)
- [ ] Command `analyze` dan `compare` (CLI table output)
- [ ] README dengan demo GIF + 1 studi kasus nyata (audit terhadap sesi ber-Caveman/RTK milik sendiri)
- [ ] Deteksi & warning soal retensi log 30 hari

### 7.2 Fase 2 (v0.2–0.3)

- [ ] Dukungan Codex CLI dan Cursor
- [ ] `export` ke JSON/CSV untuk analisis lanjutan
- [ ] Reasoning token breakdown (jika tersedia di transcript)
- [ ] HTML report generator (opsional, bukan default)
- [ ] Update pricing via file config manual

### 7.3 Fase 3 (v1.0+, tergantung traksi)

- [ ] Plugin/integrasi resmi dengan tool kompresi populer (jika mereka membuka API/hook untuk audit pihak ketiga)
- [ ] Team-level aggregation (opsional, tetap privacy-first — data tidak dikirim ke server pihak ketiga tanpa consent eksplisit)

---

## 8. Metrik Keberhasilan

| Metrik | Target 30 hari pasca-launch | Target 90 hari |
|---|---|---|
| GitHub stars | 200+ | 1,000+ |
| Kontributor eksternal (PR merged) | 1+ | 5+ |
| Disebut di README/dokumentasi tool lain (RTK/Caveman/Ponytail) | 0 (aspirational) | 1+ |
| Mention organik di HN/Reddit tanpa dipromosikan ulang | - | 1+ thread signifikan |

Catatan: hindari menjadikan stars sebagai tujuan utama produk — treat sebagai indikator lagging dari kegunaan nyata, sesuai prinsip yang sudah dibahas sebelumnya.

---

## 9. Risiko & Mitigasi

| Risiko | Dampak | Mitigasi |
|---|---|---|
| Format JSONL Claude Code berubah antar versi (resmi dikonfirmasi "internal, dapat berubah") | Parser rusak tiap update Claude Code | Tulis parser dengan graceful degradation (skip field tak dikenal, jangan crash); pin versi minimum yang didukung; siapkan test suite dengan sample transcript dari beberapa versi |
| Log dihapus otomatis sebelum sempat dianalisis (retensi default ~30 hari) | Data hilang, hasil audit tidak lengkap | First-run check + saran eksplisit ke pengguna untuk override retention di `settings.json` |
| Dianggap "pesaing" oleh maintainer tool kompresi, bukan mitra netral | Kehilangan peluang rekomendasi silang | Positioning komunikasi eksplisit sejak README pertama: audit independen, bukan alternatif kompresi; kontak baik-baik ke maintainer RTK/Caveman/Ponytail sebelum/saat launch |
| Angka yang dihasilkan TokenTruth sendiri disebut bias/tidak akurat | Kredibilitas hancur — ini fatal karena kredibilitas adalah value proposition inti | Metodologi perhitungan harus dipublikasikan terbuka (bukan black box), reproducible dari data transcript apa adanya, tidak ada estimasi tersembunyi |
| Nama "TokenTruth" collision di masa depan atau kurang SEO-distinct | Sulit ditemukan / disalahpahami | Cek ketersediaan GitHub org, npm/crates.io, domain sebelum commit; siapkan tagline pembeda kuat di README |

---

## 10. Strategi Peluncuran (ringkas)

1. **Pra-launch:** Jalankan TokenTruth terhadap sesi sendiri yang sudah pakai Caveman/RTK/Ponytail; kumpulkan data nyata (bukan hipotetis) sebagai bahan launch post.
2. **README sebagai landing page:** satu kalimat pitch jelas, demo GIF, cara install 1 baris (`cargo install tokentruth` atau `curl | sh`), metodologi terbuka.
3. **Launch post (Show HN + Reddit r/ClaudeAI, r/LocalLLaMA):** judul jujur dan berbasis data, contoh arah: *"Show HN: I audited real token savings from Caveman/RTK/Ponytail on my own sessions"*.
4. **Minggu pertama:** respons cepat ke semua issue; siap menerima kritik metodologi dengan terbuka (ini pasar yang sensitif terhadap klaim tidak akurat — kredibilitas harus dijaga ketat).
5. **Follow-up:** ajukan PR/issue sopan ke repo RTK/Caveman/Ponytail menawarkan diri sebagai referensi verifikasi independen di README mereka.

---

## 11. Pertanyaan Terbuka

- Apakah perlu dukungan multi-session aggregation di MVP, atau cukup single-session dulu?
- Apakah "reasoning tokens" konsisten tersedia di transcript Claude Code untuk semua model, atau perlu fallback?
- Lisensi: MIT/Apache 2.0 (ikuti konvensi RTK) — perlu diputuskan sebelum commit pertama.
- Nama final: konfirmasi ketersediaan `tokentruth` di GitHub, crates.io, dan domain sebelum mulai coding.

---

*Dokumen ini adalah draft kerja — revisi diharapkan seiring temuan teknis saat riset format JSONL lebih dalam dan validasi awal dari calon pengguna.*
