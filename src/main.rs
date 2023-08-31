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

        /// resize arguments
        #[arg(long)]
        resize: Option<String>,
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

enum CriteriaField {
    AppId,
    Class,
}

async fn show(
    criteria: &Criteria,
    exec: &String,
    resize: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    let criteria_field: CriteriaField;
    let criteria_value: &String;

    let criteria = match criteria.app_id {
        Some(ref app_id_value) => {
            criteria_field = CriteriaField::AppId;
            criteria_value = app_id_value;
            format!("app_id={app_id_value}")
        }
        None => {
            let class_value = criteria
                .class
                .as_ref()
                .expect("--class must be specified if --app-id is not specified");
            criteria_field = CriteriaField::Class;
            criteria_value = class_value;
            format!("class={class_value}")
        }
    };

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
    let mut is_target_showing_on_focused = false;

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
                let is_scratch = showing_scratches.contains(&&node.id);
                let is_target = match criteria_field {
                    CriteriaField::AppId => node
                        .app_id
                        .as_ref()
                        .is_some_and(|app_id| *app_id == *criteria_value),
                    CriteriaField::Class => {
                        node.window_properties.as_ref().is_some_and(|window_props| {
                            window_props
                                .class
                                .as_ref()
                                .is_some_and(|class| *class == *criteria_value)
                        })
                    }
                };
                is_target_showing_on_focused |= is_scratch && is_target;
                is_scratch && !is_target
            })
            .collect();

        for showing_scratch_node in showing_scratch_nodes {
            let showing_id = showing_scratch_node.id;
            connection
                .run_command(format!("[con_id={showing_id}] scratchpad show"))
                .await?;
        }
    }

    // second, try to toggle our named scratch
    let show_res = connection
        .run_command(format!("[{criteria}] scratchpad show"))
        .await?;

    // if failed to show, we need to create a new scratch
    if !show_res
        .first()
        .is_some_and(|show_res_inner| show_res_inner.is_ok())
    {
        let create_res = Command::new("sh").arg("-c").arg(exec).arg("&").status();
        match create_res {
            Err(err) => return Err(Box::new(err)),
            _ => {}
        };
    }

    // lastly, resize if needed

    if !is_target_showing_on_focused {
        match resize {
            Some(resize_arg) => {
                let move_res = connection
                    .run_command(format!("[{criteria}] move position center"))
                    .await?
                    .into_iter()
                    .next()
                    .unwrap();

                let resize_res = connection
                    .run_command(format!("[{criteria}] resize {resize_arg}"))
                    .await?
                    .into_iter()
                    .next()
                    .unwrap();

                let mut res = Ok(());

                if move_res.is_err() {
                    res = move_res;
                } else if resize_res.is_err() {
                    res = resize_res;
                }
                match res {
                    Err(err) => Err(Box::from(err)),
                    _ => Ok(()),
                }
            }
            _ => Ok(()),
        }
    } else {
        Ok(())
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let res = match &cli.command {
        Commands::Show {
            criteria,
            exec,
            resize,
        } => show(criteria, exec, resize).await,
    };

    res
}
