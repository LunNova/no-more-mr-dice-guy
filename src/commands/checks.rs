use crate::prelude::*;
use anyhow::{anyhow, Result};
use serenity::framework::standard::{macros::check, Args, CommandOptions, Reason};
use serenity::{model::prelude::*, prelude::*};

#[check]
#[name = "ManageRolesHigh"]
async fn manage_roles_high(
	ctx: &Context,
	msg: &Message,
	_: &mut Args,
	_: &CommandOptions,
) -> Result<(), Reason> {
	match check_manage_roles_high(ctx, msg).await {
		Ok(None) => Ok(()),
		Ok(Some(reason)) => Err(reason),
		Err(err) => {
			warn!(
				"Manage roles check failed for {} due to {:?}",
				&msg.author.name, err
			);
			Err(Reason::UserAndLog {
				user: msg.author.name.clone(),
				log: err.to_string(),
			})
		}
	}
}

// Had to reimplement permissions checks as they don't work when the GUILD_MEMBERS intent isn't used
// https://github.com/serenity-rs/serenity/issues/888
async fn check_manage_roles_high(ctx: &Context, msg: &Message) -> Result<Option<Reason>> {
	Ok(match msg.guild(&ctx) {
		None => Some(Reason::UserAndLog {
			user: msg.author.name.clone(),
			log: "Not in a guild".to_string(),
		}),
		Some(guild) => {
			if guild.owner_id == msg.author.id {
				None
			} else {
				let mut allowed = false;

				let mut bot_highest_manage_roles_permission = None;
				{
					let bot_member = guild.member(&ctx, ctx.cache.current_user_id()).await?;
					for x in &bot_member.roles {
						let role = guild
							.roles
							.get(x)
							.ok_or_else(|| anyhow!("Couldn't find role {} in {}", x, guild.name))?;
						if role.has_permission(Permissions::ADMINISTRATOR)
							|| role.has_permission(Permissions::MANAGE_ROLES)
						{
							match bot_highest_manage_roles_permission {
								None => bot_highest_manage_roles_permission = Some(role.position),
								Some(pos) => {
									bot_highest_manage_roles_permission =
										Some(std::cmp::max(pos, role.position))
								}
							}
						}
					}
				}

				let bot_highest_manage_roles_permission = match bot_highest_manage_roles_permission
				{
					None => return Err(anyhow!("Bot must have manage roles permission")),
					Some(highest) => highest,
				};

				let member = guild.member(&ctx, msg.author.id).await?;

				let mut highest_manage_roles_permission = None;
				for x in &member.roles {
					let role = guild
						.roles
						.get(x)
						.ok_or_else(|| anyhow!("Couldn't find role {} in {}", x, guild.name))?;
					if role.has_permission(Permissions::ADMINISTRATOR) {
						return Ok(None);
					}
					if role.has_permission(Permissions::MANAGE_ROLES) {
						match highest_manage_roles_permission {
							None => highest_manage_roles_permission = Some(role.position),
							Some(pos) => {
								highest_manage_roles_permission =
									Some(std::cmp::max(pos, role.position))
							}
						}
					}
				}

				match highest_manage_roles_permission {
					Some(highest_manage_roles_permission) => {
						if highest_manage_roles_permission >= bot_highest_manage_roles_permission {
							allowed = true
						}
					}
					None => return Err(anyhow!("User must have manage roles permission")),
				}

				info!("Failed manage roles check");

				if allowed {
					None
				} else {
					Some(Reason::UserAndLog {
						user: msg.author.name.clone(),
						log: "Manage roles permission on a role below manage roles permission of this bot".to_string()
					})
				}
			}
		}
	})
}
