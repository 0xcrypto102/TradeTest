use anchor_lang::prelude::*;

declare_id!("CxHzmPBWHnpb5sFYcTde88DMEym63dybWMTH6smu3tkN");

#[program]
pub mod trade {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
