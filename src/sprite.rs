use ggez::{graphics, Context, GameResult};
use graphics::Image;
pub struct AllSprite {
    pub hori: graphics::spritebatch::SpriteBatch,
    pub vert: graphics::spritebatch::SpriteBatch,
    pub diag_ne: graphics::spritebatch::SpriteBatch,
    pub diag_se: graphics::spritebatch::SpriteBatch,
    pub agent: graphics::spritebatch::SpriteBatch,
}

impl AllSprite {
    pub fn new(ctx: &mut Context) -> GameResult<AllSprite> {
        let mut hori_img = Image::from_rgba8(
            ctx,
            3,
            3,
            &[
                123, 123, 123, 0, 123, 123, 123, 0, 123, 123, 123, 0, 123, 123, 123, 255, 123, 123,
                123, 255, 123, 123, 123, 255, 123, 123, 123, 0, 123, 123, 123, 0, 123, 123, 123, 0,
            ],
        )?;

        hori_img.set_filter(ggez::graphics::FilterMode::Nearest);

        let hori = graphics::spritebatch::SpriteBatch::new(hori_img);

        let mut vert_img = Image::from_rgba8(
            ctx,
            3,
            3,
            &[
                123, 123, 123, 0, 123, 123, 123, 255, 123, 123, 123, 0, 123, 123, 123, 0, 123, 123,
                123, 255, 123, 123, 123, 0, 123, 123, 123, 0, 123, 123, 123, 255, 123, 123, 123, 0,
            ],
        )?;
        vert_img.set_filter(ggez::graphics::FilterMode::Nearest);
        let vert = graphics::spritebatch::SpriteBatch::new(vert_img);
        let mut diag_img_ne = Image::from_rgba8(
            ctx,
            3,
            3,
            &[
                123, 123, 123, 0, 123, 123, 123, 0, 123, 123, 123, 255, 123, 123, 123, 0, 123, 123,
                123, 255, 123, 123, 123, 0, 123, 123, 123, 255, 123, 123, 123, 0, 123, 123, 123, 0,
            ],
        )?;
        diag_img_ne.set_filter(ggez::graphics::FilterMode::Nearest);
        let diag_ne = graphics::spritebatch::SpriteBatch::new(diag_img_ne);

        let mut diag_img_se = Image::from_rgba8(
            ctx,
            3,
            3,
            &[
                123, 123, 123, 255, 123, 123, 123, 0, 123, 123, 123, 0, 123, 123, 123, 0, 123, 123,
                123, 255, 123, 123, 123, 0, 123, 123, 123, 0, 123, 123, 123, 0, 123, 123, 123, 255,
            ],
        )?;
        diag_img_se.set_filter(ggez::graphics::FilterMode::Nearest);
        let diag_se = graphics::spritebatch::SpriteBatch::new(diag_img_se);

        let mut agent_vec = Vec::with_capacity(4 * 4 * 4);
        for i in 0..16 {
            agent_vec.push(255);
            agent_vec.push(255);
            agent_vec.push(0);
            if [0, 3, 12, 15].contains(&i) {
                agent_vec.push(0);
            } else {
                agent_vec.push(255);
            }
        }
        let mut agent_img = Image::from_rgba8(ctx, 4, 4, &agent_vec)?;
        agent_img.set_filter(ggez::graphics::FilterMode::Linear);
        let agent = graphics::spritebatch::SpriteBatch::new(agent_img);

        Ok(AllSprite {
            hori,
            vert,
            diag_ne,
            diag_se,
            agent,
        })
    }

    pub fn clear(&mut self) {
        self.hori.clear();
        self.vert.clear();
        self.diag_ne.clear();
        self.diag_se.clear();
    }
}
