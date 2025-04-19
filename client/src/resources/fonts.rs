use bevy::asset::{AssetServer, Handle};
use bevy::prelude::{Commands, Deref, DerefMut, Res, Resource};
use bevy::text::Font;

pub fn load_fonts(mut cmds: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server.load::<Font>(FontAsset::SpaceGrotesk.as_ref());
    cmds.insert_resource(FontSpaceGrotesk(font_handle));
}

#[derive(Resource, Deref, DerefMut)]
pub struct FontSpaceGrotesk(pub Handle<Font>);

pub enum FontAsset {
    SpaceGrotesk,
}

impl AsRef<str> for FontAsset {
    fn as_ref(&self) -> &str {
        match self {
            FontAsset::SpaceGrotesk => {
                "embedded://tic_tac_toe/../assets/fonts/SpaceGrotesk-Medium.ttf"
            }
        }
    }
}
