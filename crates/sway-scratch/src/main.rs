use clap::Parser;
use std::error::Error;
use sway_scratch::*;
use swayipc_async::Connection;

enum CriteriaField {
    AppId,
    Class,
}

async fn show(
    criteria: &Criteria,
    exec: &String,
    resize: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    // "target" is the scratch we want to show or hide
    let target_criteria_field: CriteriaField;
    let target_criteria_value: &String;
    let target_criteria = match criteria.app_id {
        Some(ref app_id_value) => {
            target_criteria_field = CriteriaField::AppId;
            target_criteria_value = app_id_value;
            format!("app_id={app_id_value}")
        }
        None => {
            let class_value = criteria
                .class
                .as_ref()
                .expect("--class must be specified if --app-id is not specified");
            target_criteria_field = CriteriaField::Class;
            target_criteria_value = class_value;
            format!("class={class_value}")
        }
    };

    let mut connection = Connection::new().await?;
    let tree = connection.get_tree().await?;

    // commands are semicolon separated
    // each command can perform multiple tasks which are comma separated
    // each task can only apply to one criteria specification
    let mut commands_to_run: Vec<Vec<String>> = Vec::new();

    // whether target is showing on focused workspace
    let mut is_target_showing_on_focused = false;

    // scratch has its own special output node
    let scratch_output = tree
        .nodes
        .iter()
        .find(|node| match node.name {
            Some(ref name) => name == "__i3",
            _ => false,
        })
        .expect("scratch output not found");

    // scratch has its own special workspace node on its special output node
    let scratch_workspace = scratch_output
        .nodes
        .iter()
        .find(|node| match node.name {
            Some(ref name) => name == "__i3_scratch",
            _ => false,
        })
        .expect("scratch workspace not found");

    // first, if a showing scratch on current output and it is not our target scratch, toggle it
    // if any other scratches are on different output, we don't care and it can stay showing

    // focus contains all showing and hidden scratch ids
    let showing_scratch_ids: Vec<i64> = scratch_workspace
        .focus
        .iter()
        .filter(|focus| {
            !scratch_workspace
                // floating contains all hidden scratches
                .floating_nodes
                .iter()
                .any(|floating| floating.id == **focus)
        })
        .copied()
        .collect();

    // if any showing, we need to check which output they are on
    if !showing_scratch_ids.is_empty() {
        let focused_workspace = tree
            .nodes
            .iter()
            .find_map(|output| {
                output.nodes.iter().find(|workspace| {
                    workspace.focused
                        || workspace.nodes.iter().any(|node| node.focused)
                        || workspace.floating_nodes.iter().any(|node| node.focused)
                })
            })
            .expect("focused workspace not found");

        let nontarget_showing_scratch_ids: Vec<i64> = focused_workspace
            .floating_nodes
            .iter()
            .filter(|node| {
                let is_scratch = showing_scratch_ids.contains(&node.id);
                let is_target = match target_criteria_field {
                    CriteriaField::AppId => node
                        .app_id
                        .as_ref()
                        .is_some_and(|app_id| *app_id == *target_criteria_value),
                    CriteriaField::Class => {
                        node.window_properties.as_ref().is_some_and(|window_props| {
                            window_props
                                .class
                                .as_ref()
                                .is_some_and(|class| *class == *target_criteria_value)
                        })
                    }
                };
                is_target_showing_on_focused |= is_scratch && is_target;
                is_scratch && !is_target
            })
            .map(|node| node.id)
            .collect();

        commands_to_run.extend(nontarget_showing_scratch_ids.iter().map(|id| {
            let mut nontarget_hide_cmd: Vec<String> = Vec::new();
            nontarget_hide_cmd.push(format!("[con_id={id}] scratchpad show"));
            nontarget_hide_cmd
        }));
    }

    // second, try to toggle our named scratch

    let mut target_cmd: Vec<String> = Vec::new();
    target_cmd.push(format!("[{target_criteria}]"));
    target_cmd.push("scratchpad show".to_string());

    // third, include resize/move if needed
    // if showing on focused, it will be hidden and the resize/move will fail
    if !is_target_showing_on_focused {
        if let Some(resize_arg) = resize {
            target_cmd.push(format!("resize {resize_arg}"));
            target_cmd.push("move position center".to_string());
        }
    }

    commands_to_run.push(target_cmd);

    let swaymsg = commands_to_run
        .iter()
        .map(|inner| inner.join(","))
        .collect::<Vec<String>>()
        .join(";");

    let show_res = connection.run_command(swaymsg).await?;

    // if success, nothing else to do
    if show_res.last().unwrap().is_ok() {
        return Ok(());
    }

    // scratch might not be floating anymore
    // if so, move it back to scratch workspace
    let move_scratchpad_res = connection
        .run_command(format!("[{target_criteria}] move scratchpad"))
        .await?;

    if move_scratchpad_res.first().unwrap().is_ok() {
        return Ok(());
    }

    // otherwise we need to exec
    match connection
        .run_command(format!("exec {exec}"))
        .await?
        .into_iter()
        .next()
        .unwrap()
    {
        Err(err) => Err(Box::new(err)),
        _ => Ok(()),
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Show {
            criteria,
            exec,
            resize,
        } => show(criteria, exec, resize).await,
    }
}
