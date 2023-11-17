use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

const DANGEROUS_ROLE_PERCENTAGE: f32 = 0.5;
const DANGEROUS_ROLE_PERMISSIONS: &[u64] = &[
    Permissions::KICK_MEMBERS.bits(),
    Permissions::BAN_MEMBERS.bits(),
    Permissions::ADMINISTRATOR.bits(),
    Permissions::MANAGE_CHANNELS.bits(),
    Permissions::MANAGE_GUILD.bits(),
    Permissions::VIEW_AUDIT_LOG.bits(),
    Permissions::MANAGE_MESSAGES.bits(),
    Permissions::VIEW_GUILD_INSIGHTS.bits(),
    Permissions::MUTE_MEMBERS.bits(),
    Permissions::DEAFEN_MEMBERS.bits(),
    Permissions::MOVE_MEMBERS.bits(),
    Permissions::MANAGE_NICKNAMES.bits(),
    Permissions::MANAGE_ROLES.bits(),
    Permissions::MANAGE_WEBHOOKS.bits(),
    Permissions::MANAGE_EVENTS.bits(),
    Permissions::MANAGE_THREADS.bits(),
    Permissions::MODERATE_MEMBERS.bits(),
];

#[command]
#[aliases("full")]
async fn full(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let role_id = match args.single::<RoleId>() {
        Ok(role_id) => role_id,
        Err(_) => {
            msg.reply(ctx, "Please provide a role ID.").await?;
            return Ok(());
        }
    };

    let guild = match msg.guild(ctx) {
        Some(guild) => guild,
        None => {
            msg.reply(ctx, "Guild not found in cache.").await?;
            return Ok(());
        }
    };

    let role = match guild.roles.get(&role_id) {
        Some(role) => role,
        None => {
            msg.reply(ctx, "Role not found in cache.").await?;
            return Ok(());
        }
    };

    let members = guild.members.values().cloned().collect::<Vec<Member>>();
    let report = generate_dangerous_permissions_report(role, &members);

    send_dangerous_permissions_report(ctx, msg, &report).await
}

#[command]
#[aliases("info")]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = match msg.guild(ctx) {
        Some(guild) => guild,
        None => {
            msg.reply(ctx, "Guild not found in cache.").await?;
            return Ok(());
        }
    };

    let mut dangerous_roles_info = Vec::new();
    let members = guild.members.values().cloned().collect::<Vec<Member>>();

    for role in guild.roles.values() {
        let report = generate_dangerous_permissions_report(role, &members);
        if (report.has_dangerous_permissions && report.over_percentage) || report.false_everyone_ping {
            dangerous_roles_info.push(format!("{} ({}) ({}%)", role.name, role.id, report.percentage * 100.0));
        }
    }

    msg.channel_id.send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Server Role Info");
            e.description(format!("Found {} roles in the server.", guild.roles.len()));
            e.field(
                "Dangerous Roles Over Percentage",
                if !dangerous_roles_info.is_empty() {
                    dangerous_roles_info.join(", ")
                } else {
                    "None".to_string()
                },
                false,
            );
            e
        })
    }).await?;

    Ok(())
}

async fn send_dangerous_permissions_report(
    ctx: &Context,
    msg: &Message,
    report: &DangerousPermissionsReport<'_>,
) -> CommandResult {
    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.title(format!("Role Report for {}", report.role.name));
                e.description(format!(
                    "This role has been assigned to {} users ({}% of the server).",
                    report.users.len(),
                    report.percentage * 100.0
                ));
                e.field(
                    "Dangerous Permissions",
                    if report.has_dangerous_permissions {
                        report
                            .dangerous_permissions
                            .iter()
                            .map(|&perm| format!("`{}`", perm))
                            .collect::<Vec<String>>()
                            .join(", ")
                    } else {
                        "None".to_string()
                    },
                    false,
                );
                e.field(
                    "Over Dangerous Role Percentage",
                    if report.over_percentage { "Yes" } else { "No" },
                    false,
                );
                e.field(
                    "False Everyone Ping",
                    if report.false_everyone_ping { "Yes" } else { "No" },
                    false,
                );
                e.field("Role ID", format!("`{}`", report.role.id), false);
                e.field(
                    "Role Permissions",
                    format!("`{}`", report.permissions.bits()),
                    false,
                );

                e.footer(|f| {
                    f.text(format!("Shard ID: {}", ctx.shard_id))
                });
                e
            })
        })
        .await?;

    Ok(())
}

struct DangerousPermissionsReport<'a> {
    role: &'a Role,
    percentage: f32,
    over_percentage: bool,
    permissions: Permissions,
    has_dangerous_permissions: bool,
    dangerous_permissions: Vec<&'static str>,
    users: Vec<&'a User>,
    false_everyone_ping: bool,
}

fn generate_dangerous_permissions_report<'a>(
    role: &'a Role,
    members: &'a Vec<Member>,
) -> DangerousPermissionsReport<'a> {
    let num_members_with_role = members
        .iter()
        .filter(|m| m.roles.contains(&role.id))
        .count();

    let percentage = num_members_with_role as f32 / members.len() as f32;
    let over_percentage = percentage >= DANGEROUS_ROLE_PERCENTAGE;

    let permissions = role.permissions;

    // Fetch all permission names
    let all_permission_names = permissions.get_permission_names();

    // Filter out the dangerous permissions
    let dangerous_permissions = DANGEROUS_ROLE_PERMISSIONS
        .iter()
        .filter_map(|&perm| {
            let perm_instance = Permissions::from_bits_truncate(perm);
            all_permission_names
                .iter()
                .find(|&&name| {
                    permissions.contains(perm_instance) && perm_instance.to_string() == name
                })
                .cloned()
        })
        .collect::<Vec<&'static str>>();

    let has_dangerous_permissions = !dangerous_permissions.is_empty();

    // Fetch all users with the role
    let users = members
        .iter()
        .filter(|m| m.roles.contains(&role.id))
        .map(|m| &m.user)
        .collect::<Vec<&User>>();

    // If the role is over the percentage and pingable, it's considered dangerous
    let false_everyone_ping = role.mentionable && over_percentage;

    DangerousPermissionsReport {
        role,
        percentage,
        over_percentage,
        permissions,
        has_dangerous_permissions,
        dangerous_permissions,
        users,
        false_everyone_ping,
    }
}
