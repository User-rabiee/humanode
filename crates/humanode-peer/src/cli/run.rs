//! The main entrypoint.

use humanode_runtime::Block;
use sc_service::PartialComponents;

use crate::service;

use super::{Root, Subcommand};

/// Parse command line arguments and run the requested operation.
pub async fn run() -> sc_cli::Result<()> {
    let root: Root = sc_cli::SubstrateCli::from_args();

    match &root.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&root),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .async_run(|config| async move {
                    let PartialComponents {
                        client,
                        task_manager,
                        import_queue,
                        ..
                    } = service::new_partial(&config)?;
                    Ok((cmd.run(client, import_queue), task_manager))
                })
                .await
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .async_run(|config| async move {
                    let PartialComponents {
                        client,
                        task_manager,
                        ..
                    } = service::new_partial(&config)?;
                    Ok((cmd.run(client, config.database), task_manager))
                })
                .await
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .async_run(|config| async move {
                    let PartialComponents {
                        client,
                        task_manager,
                        ..
                    } = service::new_partial(&config)?;
                    Ok((cmd.run(client, config.chain_spec), task_manager))
                })
                .await
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .async_run(|config| async move {
                    let PartialComponents {
                        client,
                        task_manager,
                        import_queue,
                        ..
                    } = service::new_partial(&config)?;
                    Ok((cmd.run(client, import_queue), task_manager))
                })
                .await
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = root.create_humanode_runner(cmd)?;
            runner
                .async_run(|config| async move {
                    let PartialComponents {
                        client,
                        task_manager,
                        backend,
                        ..
                    } = service::new_partial(&config)?;
                    Ok((cmd.run(client, backend), task_manager))
                })
                .await
        }
        Some(Subcommand::Benchmark(cmd)) => {
            if cfg!(feature = "runtime-benchmarks") {
                let runner = root.create_humanode_runner(cmd)?;
                runner.sync_run(|config| cmd.run::<Block, service::Executor>(config))
            } else {
                Err(
                    "Benchmarking wasn't enabled when building the node. You can enable it with \
                     `--features runtime-benchmarks`."
                        .into(),
                )
            }
        }
        None => {
            let runner = root.create_humanode_runner(&root.run)?;
            sc_cli::print_node_infos::<Root>(runner.config());
            runner
                .run_node(|config| async move {
                    service::new_full(config)
                        .await
                        .map_err(sc_cli::Error::Service)
                })
                .await
        }
    }
}