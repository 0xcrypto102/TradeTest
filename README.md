# Get Started

Please make sure that you install the Solana/Anchor/Rust on your development environment

## How to build

    anchor build

## How to deploy

    anchor deploy

## How to test

  anchor run test

## Logic

- Init the program
- Deposit SOL to vault by owner
- Deposit USDC to token vault by owner
- Allow users to deposit SOL and withdraw USDC (based on SOL price from Pyth)
- Allow users to deposit USDC and withdraw SOL (based on SOL price from Pyth)
