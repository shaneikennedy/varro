import { run } from 'pierre';

export default async () => {
  const cwd = "lib";
  await run(`apt update -y && apt install -y pkg-config build-essential libssl-dev`)
  await run(`cargo build --all-features`, { cwd });
  await run(`cargo test --all-features`, { cwd });
  await run(`cargo fmt --all -- --check`, { cwd });
  await run(`cargo clippy --all-targets --all-features -- -D warnings`, { cwd });
};
