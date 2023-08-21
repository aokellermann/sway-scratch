use clap::{Args, Parser, Subcommand};
use std::error::Error;
use std::process::Command;
use swayipc_async::{Connection, Node};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// show named scratchpad
    Show {
        /// criteria
        #[command(flatten)]
        criteria: Criteria,

        /// command to create a new scratchpad
        #[arg(long)]
        exec: String,
    },
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct Criteria {
    /// scratchpad app_id
    #[arg(long)]
    app_id: Option<String>,

    /// scratchpad class
    #[arg(long)]
    class: Option<String>,
}

async fn show(criteria: &Criteria, exec: &String) -> Result<(), Box<dyn Error>> {
    let mut connection = Connection::new().await?;
    let tree = connection.get_tree().await?;

    // first, hide any scratchpads currently showing
    let scratch_output = tree
        .nodes
        .iter()
        .find(|node| match node.name {
            Some(ref name) => name == "__i3",
            _ => false,
        })
        .expect("scratch output not found");

    let scratch_workspace = scratch_output
        .nodes
        .iter()
        .find(|node| match node.name {
            Some(ref name) => name == "__i3_scratch",
            _ => false,
        })
        .expect("scratch workspace not found");

    // first, if a showing scratch on current output and it is not our target scratch, toggle it

    // focus contains all scratch ids
    // floating contains all hidden scratches

    let showing_scratches: Vec<&i64> = scratch_workspace
        .focus
        .iter()
        .filter(|focus| {
            !scratch_workspace
                .floating_nodes
                .iter()
                .any(|floating| floating.id == **focus)
        })
        .collect();

    if !showing_scratches.is_empty() {
        let mut focused_workspace_opt: Option<&Node> = None;
        for output in tree.nodes.iter() {
            for workspace in output.nodes.iter() {
                if workspace.focused
                    || workspace.nodes.iter().any(|node| node.focused)
                    || workspace.floating_nodes.iter().any(|node| node.focused)
                {
                    focused_workspace_opt = Some(workspace);
                }
            }
        }

        let focused_workspace = focused_workspace_opt.expect("focused workspace not found");

        let showing_scratch_nodes: Vec<&Node> = focused_workspace
            .floating_nodes
            .iter()
            .filter(|node| {
                showing_scratches.contains(&&node.id)
                    && node
                        .app_id
                        .as_ref()
                        .is_some_and(|app_id| *app_id != *app_id)
            })
            .collect();

        for showing_scratch_node in showing_scratch_nodes {
            let showing_app_id = showing_scratch_node.app_id.clone().unwrap();
            connection
                .run_command(format!("[app_id={showing_app_id}] scratchpad show"))
                .await?;
        }
    }

    let criteria = match criteria.app_id {
        Some(ref app_id_value) => format!("app_id={app_id_value}"),
        None => {
            let class_value = criteria
                .class
                .as_ref()
                .expect("--class must be specified if --app-id is not specified");
            format!("class={class_value}")
        }
    };
    let show_res = connection
        .run_command(format!("[{criteria}] scratchpad show"))
        .await?;

    // second, try to toggle our named scratch
    let show_success = match show_res.first().unwrap() {
        Ok(_) => true,
        _ => false,
    };

    // if success, nothing else to do
    if show_success {
        return Ok(());
    }

    // otherwise we need to create a new scratch
    match Command::new("sh").arg("-c").arg(exec).arg("&").status() {
        Ok(_) => Ok(()),
        Err(err) => Err(Box::new(err)),
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let res: Result<(), Box<dyn Error>> = match &cli.command {
        Commands::Show { criteria, exec } => show(criteria, exec).await,
    };

    res
}
