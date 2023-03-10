/*
    Copyright © 2023 Province of British Columbia
    https://digital.gov.bc.ca/digital-trust
*/
use crate::{
    command_executor::{
        Command, CommandContext, CommandMetadata, CommandParams, DynamicCompletionType,
    },
    params_parser::ParamParser,
};

pub mod detach_command {
    use super::*;
    use crate::tools::wallet::wallet_config::WalletConfig;

    command!(
        CommandMetadata::build("detach", "Detach wallet from Indy CLI")
            .add_main_param_with_dynamic_completion(
                "name",
                "Identifier of the wallet",
                DynamicCompletionType::Wallet
            )
            .add_example("wallet detach wallet1")
            .finalize()
    );

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx: {:?} params {:?}", ctx, secret!(params));

        let id = ParamParser::get_str_param("name", params)?;

        let config = WalletConfig::read(id)
            .map_err(|_| println_err!("Wallet \"{}\" isn't attached to CLI", id))?;

        if let Some(wallet) = ctx.get_opened_wallet() {
            if wallet.name == id {
                println_err!("Wallet \"{}\" is opened", id);
                return Err(());
            }
        }

        config
            .delete()
            .map_err(|err| println_err!("Cannot delete \"{}\" config file: {:?}", id, err))?;

        println_succ!("Wallet \"{}\" has been detached", id);

        trace!("execute << ");
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::commands::{setup, tear_down};

    mod detach {
        use super::*;
        use crate::{
            tools::wallet::Wallet,
            wallet::tests::{
                attach_wallet, close_and_delete_wallet, create_and_open_wallet, create_wallet,
                delete_wallet, WALLET,
            },
        };

        #[test]
        pub fn detach_works() {
            let ctx = setup();
            create_wallet(&ctx);
            {
                let cmd = detach_command::new();
                let mut params = CommandParams::new();
                params.insert("name", WALLET.to_string());
                cmd.execute(&CommandContext::new(), &params).unwrap();
            }

            let wallets = Wallet::list();
            assert_eq!(0, wallets.len());

            attach_wallet(&ctx);
            delete_wallet(&ctx);
            tear_down();
        }

        #[test]
        pub fn detach_works_for_not_attached() {
            let ctx = setup();

            let cmd = detach_command::new();
            let mut params = CommandParams::new();
            params.insert("name", WALLET.to_string());
            cmd.execute(&ctx, &params).unwrap_err();

            tear_down();
        }

        #[test]
        pub fn detach_works_for_opened() {
            let ctx = setup();

            create_and_open_wallet(&ctx);
            {
                let cmd = detach_command::new();
                let mut params = CommandParams::new();
                params.insert("name", WALLET.to_string());
                cmd.execute(&ctx, &params).unwrap_err();
            }
            close_and_delete_wallet(&ctx);
            tear_down();
        }
    }
}
