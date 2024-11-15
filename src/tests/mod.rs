#[cfg(test)]
mod tests {
    use mollusk_svm::{program, Mollusk};
    use solana_sdk::{
        account::{
            AccountSharedData, 
            WritableAccount
        }, instruction::{
            AccountMeta, 
            Instruction
        }, program_option::COption, program_pack::Pack, 
        pubkey::Pubkey,
        pubkey
    };
    
    const AMOUNT_TO_RAISE: u64 = 10_000_000;
    const TODAY_TIMESTAMP: i64 = 1731704609;
    const DURATION_DAYS: u8 = 2;
    
    #[test]
    fn initialize() {

        let program_id = pubkey!("22222222222222222222222222222222222222222222");

        let mut mollusk = Mollusk::new(&program_id, "target/deploy/native_fundraiser");
        mollusk_token::token::add_program(&mut mollusk);

        let (
            token_program, 
            token_program_account
        ) = mollusk_token::token::keyed_account();

        let (
            system_program, 
            system_program_account
        ) = program::keyed_account_for_system_program();

        let maker = Pubkey::new_unique();
        let mint_to_raise = Pubkey::new_unique();
        let mut mint_to_raise_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(spl_token::state::Mint::LEN), 
            spl_token::state::Mint::LEN,
            &token_program
        );
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None
            },
            mint_to_raise_account.data_as_mut_slice()
        ).unwrap();

        let (fundraiser, _bump) = Pubkey::find_program_address(
            &[b"fundraiser", maker.as_ref()],
            &program_id
        );
        let fundraiser_account = AccountSharedData::new(0, 0, &system_program);

        let (vault, _) = Pubkey::find_program_address(
            &[b"vault", fundraiser.as_ref()],
            &program_id
        );

        let vault_account = AccountSharedData::new(
            0,
            0,
            &system_program
        );

        let data = [
            vec![0],
            AMOUNT_TO_RAISE.to_le_bytes().to_vec(),
            TODAY_TIMESTAMP.to_le_bytes().to_vec(),
            DURATION_DAYS.to_le_bytes().to_vec(),
        ].concat();

        let instruction = Instruction::new_with_bytes(
            program_id, 
            &data, 
            vec![
                AccountMeta::new(maker, true),  // writable and signer
                AccountMeta::new_readonly(mint_to_raise, false),  // readonly
                AccountMeta::new(fundraiser, false),  // writable
                AccountMeta::new(vault, false),  // writable
                AccountMeta::new_readonly(system_program, false),  // readonly
                AccountMeta::new_readonly(token_program, false),  // readonly
            ]
        );

        let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
            &instruction, 
            &vec![
                (
                    maker, 
                    AccountSharedData::new(1_000_000_000_000, 0, &Pubkey::default())
                ),
                (
                    mint_to_raise,
                    mint_to_raise_account
                ),
                (
                    fundraiser,
                    fundraiser_account
                ),
                (
                    vault,
                    vault_account
                ),
                (system_program, system_program_account),
                (token_program, token_program_account)
            ],
        );
        println!("vault: {}", vault.to_string());
        assert!(!result.program_result.is_err(), "Program execution failed: {:?}", result.program_result);    }
}