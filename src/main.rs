// Copyright (C) 2023-2025  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use leptos::prelude::*;
use leptos_router::{components::*, path};

use crate::ui::Player;

mod player;
mod remote_api;
mod ui;

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/*") view=Player/>
            </Routes>
        </Router>
    }
}

fn main() {
    mount_to_body(|| view! { <App/> });
}
