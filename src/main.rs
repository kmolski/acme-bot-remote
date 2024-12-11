// Copyright (C) 2023-2024  Krzysztof Molski
// SPDX-License-Identifier: AGPL-3.0-or-later

use leptos::*;
use leptos_router::*;

use crate::ui::Player;

mod player;
mod remote_api;
mod ui;

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/*" view=Player/>
            </Routes>
        </Router>
    }
}

fn main() {
    mount_to_body(|| view! { <App/> });
}
