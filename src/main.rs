use std::{fs::File, io::BufReader, num::NonZeroU64};

use anyhow::Result;
use futures::stream::StreamExt;
use serde::Deserialize;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard};
use twilight_http::Client;
use twilight_model::{
    application::{
        callback::{CallbackData, InteractionResponse},
        component::{button::ButtonStyle, ActionRow, Button, Component},
        interaction::Interaction,
    },
    id::ApplicationId,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Secrets {
    application_id: NonZeroU64,
    discord_token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Secrets {
        discord_token,
        application_id,
    } = serde_json::from_reader(BufReader::new(File::open("./secrets.json")?))?;
    let (shard, mut events) = Shard::builder(&discord_token, Intents::GUILDS)
        .event_types(EventTypeFlags::INTERACTION_CREATE | EventTypeFlags::GUILD_CREATE)
        .build();
    let client = Client::builder()
        .token(discord_token)
        .application_id(ApplicationId(application_id))
        .build();
    shard.start().await?;
    while let Some(event) = events.next().await {
        match event {
            Event::GuildCreate(gc) => {
                if let Some(chan) = gc.system_channel_id {
                    client
                        .create_message(chan)
                        .content("Ici c'est Sparky !")?
                        .components(&[Component::ActionRow(ActionRow {
                            components: vec![Component::Button(Button {
                                custom_id: Some("shutdown".to_owned()),
                                disabled: false,
                                emoji: None,
                                label: Some("Tire-toi".to_owned()),
                                style: ButtonStyle::Primary,
                                url: None,
                            })],
                        })])?
                        .exec()
                        .await?;
                }
            }
            Event::InteractionCreate(ic) => {
                if let Interaction::MessageComponent(mc) = ic.0 {
                    match mc.data.custom_id.as_str() {
                        "shutdown" => {
                            client
                                .interaction_callback(
                                    mc.id,
                                    &mc.token,
                                    &InteractionResponse::UpdateMessage(CallbackData {
                                        allowed_mentions: None,
                                        components: Some(vec![]),
                                        content: Some("Ok j'me casse.".to_owned()),
                                        embeds: None,
                                        flags: None,
                                        tts: None,
                                    }),
                                )
                                .exec()
                                .await?;
                            shard.shutdown();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
