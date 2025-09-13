import { run } from 'pierre';

export default async () => {
  await run(`apt update -y && apt install -y build-essential`)
  await run(`cargo build`, { cwd: "pyvarro" });
  await run(`cargo test`, { cwd: "pyvarro" });
  await run(`cargo fmt --all -- --check`, { cwd: "pyvarro" });
  await run(`cargo clippy --all-targets --all-features -- -D warnings`, { cwd: "pyvarro" });
};
