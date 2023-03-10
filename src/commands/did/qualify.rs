/*
    Copyright © 2023 Province of British Columbia
    https://digital.gov.bc.ca/digital-trust
*/
use crate::{
    command_executor::{
        Command, CommandContext, CommandMetadata, CommandParams, DynamicCompletionType,
    },
    params_parser::ParamParser,
    tools::did::Did,
};

pub mod qualify_command {
    use super::*;

    command!(CommandMetadata::build(
        "qualify",
        "Update DID stored in the wallet to make fully qualified, or to do other DID maintenance."
    )
    .add_main_param_with_dynamic_completion(
        "did",
        "Did stored in wallet",
        DynamicCompletionType::Did
    )
    .add_required_param(
        "method",
        "Method to apply to the DID. Note that `did:` prefix must be omitted`"
    )
    .add_example("did qualify VsKV7grR1BUE29mG2Fm2kX method=peer")
    .finalize());

    fn execute(ctx: &CommandContext, params: &CommandParams) -> Result<(), ()> {
        trace!("execute >> ctx {:?}, params {:?}", ctx, params);

        let wallet = ctx.ensure_opened_wallet()?;
        let did = ParamParser::get_did_param("did", params)?;
        let method = ParamParser::get_str_param("method", params)?;

        let method = if method.contains("did:") {
            &method[4..]
        } else {
            method
        };

        let full_qualified_did = Did::qualify(&wallet, &did, &method)
            .map_err(|err| println_err!("{}", err.message(None)))?;

        println_succ!("Fully qualified DID \"{}\"", full_qualified_did);

        if let Some(active_did) = ctx.get_active_did()? {
            if *active_did == did {
                ctx.set_active_did(full_qualified_did);
                println_succ!("Target DID is the same as CLI active. Active DID has been updated");
            }
        }

        trace!("execute <<");
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    mod qualify_did {
        use super::*;
        use crate::{
            commands::{setup_with_wallet, tear_down_with_wallet},
            did::tests::{new_did, use_did, DID_MY1, SEED_MY1},
        };

        const METHOD: &str = "peer";

        #[test]
        pub fn qualify_did_works() {
            let ctx = setup_with_wallet();
            new_did(&ctx, SEED_MY1);
            {
                let cmd = qualify_command::new();
                let mut params = CommandParams::new();
                params.insert("did", DID_MY1.to_string());
                params.insert("method", METHOD.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn qualify_did_works_for_active() {
            let ctx = setup_with_wallet();
            new_did(&ctx, SEED_MY1);
            use_did(&ctx, DID_MY1);
            {
                let cmd = qualify_command::new();
                let mut params = CommandParams::new();
                params.insert("did", DID_MY1.to_string());
                params.insert("method", METHOD.to_string());
                cmd.execute(&ctx, &params).unwrap();
            }
            tear_down_with_wallet(&ctx);
        }

        #[test]
        pub fn qualify_did_works_for_unknown_did() {
            let ctx = setup_with_wallet();
            {
                let cmd = qualify_command::new();
                let mut params = CommandParams::new();
                params.insert("did", DID_MY1.to_string());
                params.insert("method", METHOD.to_string());
                cmd.execute(&ctx, &params).unwrap_err();
            }
            tear_down_with_wallet(&ctx);
        }
    }
}
