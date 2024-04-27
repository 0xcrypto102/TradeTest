use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
use instructions::*;

declare_id!("CxHzmPBWHnpb5sFYcTde88DMEym63dybWMTH6smu3tkN");

#[program]
pub mod trade {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }
}


