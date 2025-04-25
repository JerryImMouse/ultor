use rand::Rng;
use serenity::all::Color;
use uuid::Uuid;

pub const RED_COLOR: Color = Color::from_rgb(255, 0, 0);

pub fn gen_random_uuid() -> Uuid {
    Uuid::from_u128(rand::rng().random::<u128>())
}

pub fn gen_random_color() -> Color {
    let mut rng = rand::rng();
    Color::from_rgb(rng.random(), rng.random(), rng.random())
}

#[macro_export]
macro_rules! try_discord_unwrap {
    // Pattern: Option<T>
    ($opt:expr, none => $none_msg:expr, $(ephemeral => $ephemeral:expr)? ) => {{
        match $opt {
            Some(v) => v,
            None => return crate::bot::commands::DiscordCommandResponse::followup_embed_response(
                $none_msg,
                None,
                Some(crate::utils::RED_COLOR),
                try_discord_unwrap!(@ephemeral $($ephemeral)?),
            ),
        }
    }};

    // Pattern: Result<T, E>
    ($res:expr, error => $err_msg:expr, log => $log_msg:expr, $(ephemeral => $ephemeral:expr)? ) => {{
        match $res {
            Ok(v) => v,
            Err(e) => {
                let err_id = crate::utils::gen_random_uuid();
                log::error!("{}. {}. Error: {}", err_id, $log_msg, e);
                return crate::bot::commands::DiscordCommandResponse::followup_embed_response(
                    $err_msg,
                    Some(&err_id.to_string()),
                    Some(crate::utils::RED_COLOR),
                    try_discord_unwrap!(@ephemeral $($ephemeral)?),
                );
            }
        }
    }};

    // Pattern: Result<Option<T>, E>
    ($resopt:expr, none => $none_msg:expr, error => $err_msg:expr, log => $log_msg:expr, $(ephemeral => $ephemeral:expr)? ) => {{
        match $resopt {
            Ok(Some(v)) => v,
            Ok(None) => return crate::bot::commands::DiscordCommandResponse::followup_embed_response(
                $none_msg,
                None,
                Some(crate::utils::RED_COLOR),
                try_discord_unwrap!(@ephemeral $($ephemeral)?),
            ),
            Err(e) => {
                let err_id = crate::utils::gen_random_uuid();
                log::error!("{}. {}. Error: {}", err_id, $log_msg, e);
                return crate::bot::commands::DiscordCommandResponse::followup_embed_response(
                    $err_msg,
                    Some(&err_id.to_string()),
                    Some(crate::utils::RED_COLOR),
                    try_discord_unwrap!(@ephemeral $($ephemeral)?),
                );
            }
        }
    }};


    (@ephemeral $e:expr) => { $e };
    (@ephemeral) => { true };
}

#[macro_export]
macro_rules! extract_discord_arg {
    // ResolvedValue::String
    (
        $opts:expr,
        $name:literal,
        String
    )   => {
        $opts.iter().find_map(|opt| match (opt.name, &opt.value) {
            ($name, serenity::all::ResolvedValue::String(i)) => Some(i.to_string()),
            _ => None,
        })
    };
    
    // ResolvedValue::*
    (
        $opts:expr,
        $name:literal,
        $dtype:ident
    ) => {
        $opts.iter().find_map(|opt| match (opt.name, &opt.value) {
            ($name, serenity::all::ResolvedValue::$dtype(i)) => Some(i),
            _ => None,
        })
    };
}