use dioxus::prelude::*;

use crate::app::layouts::admin_layout::AdminLayout;
use crate::app::pages::apps::Apps;
use crate::app::pages::skills::detail::SkillDetail;
use crate::app::pages::skills::editor::SkillEditor;
use crate::app::pages::skills::list::{Skills, SkillsList};
use crate::app::pages::skills::store::SkillsStore;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AdminLayout)]
    #[route("/")]
    Skills {},
    #[route("/skills")]
    SkillsList {},
    #[route("/skills/store")]
    SkillsStore {},
    #[route("/apps")]
    Apps {},
    #[route("/skills/:id")]
    SkillDetail { id: String },
    #[route("/skills/:id/edit")]
    SkillEditor { id: String },
}
