# INSTALLATION GUIDE: Phase 2 Foundation Modules

Follow these steps to integrate the Phase 2 foundation into your `smart-tx-stack` repository.

---

## STEP 1: Verify repo structure

Your repo should look like:
```
smart-tx-stack/
├── Cargo.toml
├── Cargo.lock (auto-generated)
├── src/
│   └── main.rs (old version)
├── .git/
├── .gitignore
└── README.md
```

---

## STEP 2: Backup your current src/main.rs

```bash
cd smart-tx-stack
cp src/main.rs src/main.rs.backup
```

---

## STEP 3: Copy Phase 2 modules

All files are in `/mnt/user-data/outputs/` (or wherever you saved them).

```bash
# Copy all module files
cp /path/to/outputs/types.rs src/
cp /path/to/outputs/config.rs src/
cp /path/to/outputs/grpc.rs src/
cp /path/to/outputs/jito.rs src/
cp /path/to/outputs/lifecycle.rs src/
cp /path/to/outputs/ai_agent.rs src/
cp /path/to/outputs/main.rs src/

# Copy environment template
cp /path/to/outputs/.env.example .
```

Verify:
```bash
ls -la src/*.rs
# Should show: ai_agent.rs, config.rs, grpc.rs, jito.rs, lifecycle.rs, main.rs, types.rs
```

---

## STEP 4: Update Cargo.toml

Replace your `Cargo.toml` with this exact spec (from Grok):

```toml
[package]
name = "smart-tx-stack"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-sdk = "2.1"
solana-client = "2.1"
jito-sdk-rust = "0.3"
yellowstone-grpc-client = "13"
yellowstone-grpc-proto = "13"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dotenv = "0.15"
tracing = "0.1"
tracing-subscriber = "0.3"
bs58 = "0.5"

[[bin]]
name = "smart-tx-stack"
path = "src/main.rs"
```

---

## STEP 5: Create your .env file

```bash
cp .env.example .env
```

Then edit `.env` with your actual values:

```bash
# Edit with your editor
nano .env
# OR
vim .env
```

Minimum required values:
```env
ENVIRONMENT=devnet
YELLOWSTONE_GRPC_ENDPOINT=http://localhost:10000
JITO_BUNDLE_ENDPOINT=https://block-engine.jito.wtf/api/v1/bundles
SOLANA_RPC_URL=https://api.devnet.solana.com
WALLET_KEYPAIR_PATH=./wallet.json
JITO_TIP_ACCOUNT=9B5X4b3XfBmrKzf7YsXwqYuvz2aLf5cuucsBiB1A6qws
DEBUG=true
```

---

## STEP 6: Test compilation

```bash
cargo check
```

Expected:
```
Checking smart-tx-stack v0.1.0 (...)
Finished `check` profile [unoptimized + debuginfo] target(s) in X.XXs
```

If you see errors, check:
1. All 7 `.rs` files are in `src/`
2. `Cargo.toml` has all dependencies
3. `.env` file exists with required values

---

## STEP 7: Run the application

```bash
cargo run
```

Expected output (Phase 2):
```
═══════════════════════════════════════════════════════════
🚀 Smart Transaction Stack v0.1.0 (Phase 2)
   Advanced Infrastructure Challenge – Superteam Nigeria
═══════════════════════════════════════════════════════════
📋 Configuration loaded
   environment: devnet
   yellowstone: http://localhost:10000
   jito: https://block-engine.jito.wtf/api/v1/bundles
   solana_rpc: https://api.devnet.solana.com
   debug: true
✅ All configuration validated
🔌 Connecting to Yellowstone gRPC...
✅ Yellowstone gRPC listener started
═══════════════════════════════════════════════════════════
👂 Listening for slot updates...
Press Ctrl+C to stop
═══════════════════════════════════════════════════════════
[... logs ...]
```

If it hangs on "Connecting to Yellowstone gRPC...", it's trying to reach your Yellowstone endpoint (expected if you haven't set it up yet).

---

## STEP 8: Commit to GitHub

```bash
# Add all changes
git add -A

# Commit with meaningful message
git commit -m "Phase 2: Foundation modules (Yellowstone gRPC + Config + Types)"

# Push to GitHub
git push origin main
```

---

## TROUBLESHOOTING

### Error: "YELLOWSTONE_GRPC_ENDPOINT not set"
**Fix**: Make sure `.env` file exists in the repo root with:
```
YELLOWSTONE_GRPC_ENDPOINT=http://localhost:10000
```

### Error: "module declaration out of order" or "mod config" not found
**Fix**: All module files must be in `src/`:
```bash
ls -la src/
# Must show: types.rs, config.rs, grpc.rs, jito.rs, lifecycle.rs, ai_agent.rs, main.rs
```

### Error: Compilation fails on dependencies
**Fix**: Update Cargo.lock:
```bash
cargo update
cargo clean
cargo build
```

### Application hangs after startup
**Reason**: Trying to connect to Yellowstone gRPC endpoint (normal if endpoint doesn't exist yet)
**Fix**: 
- For testing: Set up a local Yellowstone (advanced)
- OR: Let it hang, press Ctrl+C to gracefully shutdown
- Phase 3 will add reconnection logic

---

## WHAT HAPPENS NEXT

Once Phase 2 is confirmed working:
1. Grok will provide Phase 3 spec (Jito integration + dynamic tips)
2. Claude will generate `jito.rs` implementation
3. You'll test bundle submission on devnet
4. Continue through Phase 4 (lifecycle tracking) and Phase 5 (AI agent)

---

## FILE MANIFEST (What you should have)

```
smart-tx-stack/
├── src/
│   ├── main.rs              (entry point)
│   ├── config.rs            (config loading)
│   ├── types.rs             (data types)
│   ├── grpc.rs              (Yellowstone gRPC)
│   ├── jito.rs              (stub for Phase 3)
│   ├── lifecycle.rs         (stub for Phase 4)
│   └── ai_agent.rs          (stub for Phase 5)
├── .env                     (your secrets — DO NOT COMMIT)
├── .env.example             (template — commit this)
├── Cargo.toml               (updated with Phase 2 deps)
├── Cargo.lock               (auto-generated)
├── README.md                (your project README)
└── .git/                    (GitHub repo)
```

---

## VERIFICATION CHECKLIST

- [ ] All 7 `.rs` files copied to `src/`
- [ ] `Cargo.toml` updated with Phase 2 dependencies
- [ ] `.env` created from `.env.example` with your values
- [ ] `cargo check` passes
- [ ] `cargo run` starts without panic
- [ ] Committed to GitHub
- [ ] Ready for Phase 3

---

**Status**: Phase 2 Foundation Installed ✅

When Grok gives the go-ahead for Phase 3, you're ready.
