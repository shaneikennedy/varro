import { run } from 'pierre';

export default async () => {
  await run(`apt update -y && apt install -y build-essential`)
  await run(`cargo build`);
  await run(`cargo test`);
  await run(`cargo fmt --all -- --check`);
  await run(`cargo clippy --all-targets --all-features -- -D warnings`);
};
