use arrayref::{array_ref, array_refs};
use serde::{Deserialize, Serialize};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MerpsInstruction {
    /// Initialize a group of lending pools that can be cross margined
    ///
    /// Accounts expected by this instruction (9):
    ///
    /// TODO
    InitMerpsGroup { signer_nonce: u64, valid_interval: u8 },

    /// Initialize a merps account for a user
    ///
    /// Accounts expected by this instruction (4):
    ///
    /// 0. `[]` merps_group_ai - MerpsGroup that this merps account is for
    /// 1. `[writable]` merps_account_ai - the merps account data
    /// 2. `[signer]` owner_ai - Solana account of owner of the merps account
    /// 3. `[]` rent_ai - Rent sysvar account
    InitMerpsAccount,

    /// Deposit funds into merps account
    ///
    /// Accounts expected by this instruction (8):
    ///
    /// 0. `[writable]` merps_group_ai - MerpsGroup that this merps account is for
    /// 1. `[writable]` merps_account_ai - the merps account for this user
    /// 2. `[signer]` owner_ai - Solana account of owner of the merps account
    /// 3. `[]` root_bank_ai - RootBank owned by MerpsGroup
    /// 4. `[writable]` node_bank_ai - NodeBank owned by RootBank
    /// 5. `[writable]` vault_ai - TokenAccount owned by MerpsGroup
    /// 6. `[]` token_prog_ai - acc pointed to by SPL token program id
    /// 7. `[writable]` owner_token_account_ai - TokenAccount owned by user which will be sending the funds
    Deposit { quantity: u64 },

    /// Withdraw funds that were deposited earlier.
    ///
    /// Accounts expected by this instruction (10):
    ///
    /// TODO
    Withdraw { quantity: u64 },

    /// Add a token to a merps group
    ///
    /// Accounts expected by this instruction (7):
    ///
    /// 0. `[writable]` merps_group_ai - TODO
    /// 1. `[]` mint_ai - TODO
    /// 2. `[writable]` node_bank_ai - TODO
    /// 3. `[]` vault_ai - TODO
    /// 4. `[writable]` root_bank_ai - TODO
    /// 5. `[]` oracle_ai - TODO
    /// 6. `[signer]` admin_ai - TODO
    AddAsset,
}

impl MerpsInstruction {
    pub fn unpack(input: &[u8]) -> Option<Self> {
        let (&discrim, data) = array_refs![input, 4; ..;];
        let discrim = u32::from_le_bytes(discrim);
        Some(match discrim {
            0 => {
                let data = array_ref![data, 0, 9];
                let (signer_nonce, valid_interval) = array_refs![data, 8, 1];

                MerpsInstruction::InitMerpsGroup {
                    signer_nonce: u64::from_le_bytes(*signer_nonce),
                    valid_interval: u8::from_le_bytes(*valid_interval),
                }
            }
            1 => MerpsInstruction::InitMerpsAccount,
            2 => {
                let quantity = array_ref![data, 0, 8];
                MerpsInstruction::Deposit { quantity: u64::from_le_bytes(*quantity) }
            }
            3 => {
                let data = array_ref![data, 0, 8];
                MerpsInstruction::Withdraw { quantity: u64::from_le_bytes(*data) }
            }
            4 => MerpsInstruction::AddAsset,
            _ => {
                return None;
            }
        })
    }
    pub fn pack(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

pub fn init_merps_group(
    program_id: &Pubkey,
    merps_group_pk: &Pubkey,
    signer_pk: &Pubkey,
    admin_pk: &Pubkey,
    quote_mint_pk: &Pubkey,
    quote_vault_pk: &Pubkey,
    quote_node_bank_pk: &Pubkey,
    quote_root_bank_pk: &Pubkey,

    signer_nonce: u64,
    valid_interval: u8,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*merps_group_pk, false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::ID, false),
        AccountMeta::new_readonly(*signer_pk, false),
        AccountMeta::new_readonly(*admin_pk, true),
        AccountMeta::new_readonly(*quote_mint_pk, false),
        AccountMeta::new_readonly(*quote_vault_pk, false),
        AccountMeta::new(*quote_node_bank_pk, false),
        AccountMeta::new(*quote_root_bank_pk, false),
    ];

    let instr = MerpsInstruction::InitMerpsGroup { signer_nonce, valid_interval };

    let data = instr.pack();
    Ok(Instruction { program_id: *program_id, accounts, data })
}

pub fn init_merps_account(
    program_id: &Pubkey,
    merps_group_pk: &Pubkey,
    merps_account_pk: &Pubkey,
    owner_pk: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new_readonly(*merps_group_pk, false),
        AccountMeta::new(*merps_account_pk, false),
        AccountMeta::new_readonly(*owner_pk, true),
        AccountMeta::new_readonly(solana_program::sysvar::rent::ID, false),
    ];

    let instr = MerpsInstruction::InitMerpsAccount;
    let data = instr.pack();
    Ok(Instruction { program_id: *program_id, accounts, data })
}

pub fn deposit(
    program_id: &Pubkey,
    merps_group_pk: &Pubkey,
    merps_account_pk: &Pubkey,
    owner_pk: &Pubkey,
    root_bank_pk: &Pubkey,
    node_bank_pk: &Pubkey,
    vault_pk: &Pubkey,
    owner_token_account_pk: &Pubkey,

    quantity: u64,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*merps_group_pk, false),
        AccountMeta::new(*merps_account_pk, false),
        AccountMeta::new_readonly(*owner_pk, true),
        AccountMeta::new(*root_bank_pk, false),
        AccountMeta::new(*node_bank_pk, false),
        AccountMeta::new(*vault_pk, false),
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new(*owner_token_account_pk, false),
    ];

    let instr = MerpsInstruction::Deposit { quantity };
    let data = instr.pack();
    Ok(Instruction { program_id: *program_id, accounts, data })
}
