use ggez::{graphics, Context, GameResult};
use graphics::Image;
pub struct AllSprite {
    pub hori: graphics::spritebatch::SpriteBatch,
    pub vert: graphics::spritebatch::SpriteBatch,
    pub diag_ne: graphics::spritebatch::SpriteBatch,
    pub diag_se: graphics::spritebatch::SpriteBatch,
}

impl AllSprite {
    pub fn new(ctx: &mut Context) -> GameResult<AllSprite> {
        let mut hori_img = Image::from_rgba8(
            ctx,
            3,
            3,
            &[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        )?;

        hori_img.set_filter(ggez::graphics::FilterMode::Nearest);

        let hori = graphics::spritebatch::SpriteBatch::new(hori_img);

        let mut vert_img = Image::from_rgba8(
            ctx,
            3,
            3,
            &[
                0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 255, 0, 0, 0, 0,
            ],
        )?;
        vert_img.set_filter(ggez::graphics::FilterMode::Nearest);
        let vert = graphics::spritebatch::SpriteBatch::new(vert_img);
        let mut diag_img_ne = Image::from_rgba8(
            ctx,
            3,
            3,
            &[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0,
                0, 255, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        )?;
        diag_img_ne.set_filter(ggez::graphics::FilterMode::Nearest);
        let diag_ne = graphics::spritebatch::SpriteBatch::new(diag_img_ne);

        let mut diag_img_se = Image::from_rgba8(
            ctx,
            3,
            3,
            &[
                0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 255,
            ],
        )?;
        diag_img_se.set_filter(ggez::graphics::FilterMode::Nearest);
        let diag_se = graphics::spritebatch::SpriteBatch::new(diag_img_se);

        Ok(AllSprite {
            hori,
            vert,
            diag_ne,
            diag_se,
        })
    }

    pub fn clear(&mut self) {
        self.hori.clear();
        self.vert.clear();
        self.diag_ne.clear();
        self.diag_se.clear();
    }
}
