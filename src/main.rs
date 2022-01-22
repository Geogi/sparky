use std::{fs::File, io::BufReader, num::NonZeroU64};

use anyhow::Result;
use futures::stream::StreamExt;
use serde::Deserialize;
use twilight_cache_inmemory::InMemoryCache;
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard};
use twilight_http::Client;
use twilight_mention::Mention;
use twilight_model::{
    application::{
        callback::{CallbackData, InteractionResponse},
        command::{Command, CommandType},
        component::{button::ButtonStyle, ActionRow, Button, Component},
        interaction::{
            application_command::{CommandDataOption, CommandOptionValue},
            Interaction,
        },
    },
    channel::message::AllowedMentions,
    id::ApplicationId,
};
use twilight_util::builder::command::{CommandBuilder, RoleBuilder, StringBuilder};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Secrets {
    application_id: NonZeroU64,
    discord_token: String,
}

fn semaine() -> Command {
    CommandBuilder::new(
        "semaine".into(),
        "Crée un sondage pour un ou plusieurs jours de la semaine".into(),
        CommandType::ChatInput,
    )
    .option(StringBuilder::new("intitulé".into(), "Intitulé du sondage".into()).required(false))
    .option(RoleBuilder::new("rôle".into(), "Rôle à notifier".into()).required(false))
    .build()
}

#[tokio::main]
async fn main() -> Result<()> {
    let Secrets {
        discord_token,
        application_id,
    } = serde_json::from_reader(BufReader::new(File::open("./secrets.json")?))?;
    let (shard, mut events) = Shard::builder(&discord_token, Intents::empty())
        .event_types(EventTypeFlags::SHARD_CONNECTED | EventTypeFlags::INTERACTION_CREATE)
        .build();
    let client = Client::builder()
        .token(discord_token)
        .application_id(ApplicationId(application_id))
        .build();
    let cache = InMemoryCache::new();
    shard.start().await?;
    while let Some(event) = events.next().await {
        cache.update(&event);
        match event {
            Event::ShardConnected(_) => {
                client.set_global_commands(&[semaine()])?.exec().await?;
            }
            Event::InteractionCreate(ic) => match ic.0 {
                Interaction::MessageComponent(mc) => {}
                Interaction::ApplicationCommand(ac) => match ac.data.name.as_str() {
                    "semaine" => {
                        let title = match ac.data.options.iter().find(|cdo| &cdo.name == "intitulé")
                        {
                            Some(CommandDataOption {
                                value: CommandOptionValue::String(title),
                                ..
                            }) => Some(title),
                            _ => None,
                        };
                        let role = match ac.data.options.iter().find(|cdo| &cdo.name == "rôle") {
                            Some(CommandDataOption {
                                value: CommandOptionValue::Role(role),
                                ..
                            }) => Some(*role),
                            _ => None,
                        };
                        client
                            .interaction_callback(
                                ac.id,
                                &ac.token,
                                &InteractionResponse::ChannelMessageWithSource(CallbackData {
                                    allowed_mentions: role.and_then(|r| {
                                        Some(AllowedMentions {
                                            parse: vec![],
                                            users: vec![],
                                            roles: vec![r],
                                            replied_user: false,
                                        })
                                    }),
                                    components: Some(vec![
                                        Component::ActionRow(ActionRow {
                                            components: vec![
                                                Component::Button(Button {
                                                    custom_id: Some("lundi".into()),
                                                    disabled: false,
                                                    emoji: None,
                                                    label: Some("Lundi".into()),
                                                    style: ButtonStyle::Primary,
                                                    url: None,
                                                }),
                                                Component::Button(Button {
                                                    custom_id: Some("mardi".into()),
                                                    disabled: false,
                                                    emoji: None,
                                                    label: Some("Mardi".into()),
                                                    style: ButtonStyle::Primary,
                                                    url: None,
                                                }),
                                                Component::Button(Button {
                                                    custom_id: Some("mercredi".into()),
                                                    disabled: false,
                                                    emoji: None,
                                                    label: Some("Mercredi".into()),
                                                    style: ButtonStyle::Primary,
                                                    url: None,
                                                }),
                                                Component::Button(Button {
                                                    custom_id: Some("jeudi".into()),
                                                    disabled: false,
                                                    emoji: None,
                                                    label: Some("Jeudi".into()),
                                                    style: ButtonStyle::Primary,
                                                    url: None,
                                                }),
                                            ],
                                        }),
                                        Component::ActionRow(ActionRow {
                                            components: vec![
                                                Component::Button(Button {
                                                    custom_id: Some("vendredi".into()),
                                                    disabled: false,
                                                    emoji: None,
                                                    label: Some("Vendredi".into()),
                                                    style: ButtonStyle::Primary,
                                                    url: None,
                                                }),
                                                Component::Button(Button {
                                                    custom_id: Some("samedi".into()),
                                                    disabled: false,
                                                    emoji: None,
                                                    label: Some("Samedi".into()),
                                                    style: ButtonStyle::Primary,
                                                    url: None,
                                                }),
                                                Component::Button(Button {
                                                    custom_id: Some("dimanche".into()),
                                                    disabled: false,
                                                    emoji: None,
                                                    label: Some("Dimanche".into()),
                                                    style: ButtonStyle::Primary,
                                                    url: None,
                                                }),
                                                Component::Button(Button {
                                                    custom_id: Some("pas_disponible".into()),
                                                    disabled: false,
                                                    emoji: None,
                                                    label: Some("Pas disponible".into()),
                                                    style: ButtonStyle::Danger,
                                                    url: None,
                                                }),
                                            ],
                                        }),
                                    ]),

                                    content: None,
                                    embeds: Some(vec![EmbedBuilder::new()
                                        .title(if let Some(title) = title {
                                            format!("Sondage pour la semaine : {title}")
                                        } else {
                                            "Sondage pour la semaine".into()
                                        })
                                        .description(if let Some(role) = role {
                                            format!(
                                                "Merci de répondre, membres de {} !",
                                                role.mention()
                                            )
                                        } else {
                                            "Merci de répondre !".into()
                                        })
                                        .field(EmbedFieldBuilder::new("Lundi", "\u{200B}").inline())
                                        .field(EmbedFieldBuilder::new("Mardi", "\u{200B}").inline())
                                        .field(
                                            EmbedFieldBuilder::new("Mercredi", "\u{200B}").inline(),
                                        )
                                        .field(EmbedFieldBuilder::new("Jeudi", "\u{200B}").inline())
                                        .field(
                                            EmbedFieldBuilder::new("Vendredi", "\u{200B}").inline(),
                                        )
                                        .field(
                                            EmbedFieldBuilder::new("Samedi", "\u{200B}").inline(),
                                        )
                                        .field(
                                            EmbedFieldBuilder::new("Dimanche", "\u{200B}").inline(),
                                        )
                                        .field(
                                            EmbedFieldBuilder::new("Pas disponible", "\u{200B}")
                                                .inline(),
                                        )
                                        .build()?]),
                                    flags: None,
                                    tts: None,
                                }),
                            )
                            .exec()
                            .await?;
                    }
                    _ => {}
                },
                _ => (),
            },
            _ => {}
        }
    }
    Ok(())
}
