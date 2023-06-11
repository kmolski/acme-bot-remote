use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    let on_resume = |_| {};
    let on_pause = |_| {};
    let on_stop = |_| {};

    html! {
        <div>
            <button onclick={on_resume}>{ "Resume" }</button>
            <button onclick={on_pause}>{ "Pause" }</button>
            <button onclick={on_stop}>{ "Stop" }</button>
        </div>
    }
}

fn main() {
    //yew::Renderer::<App>::new().render();
}
