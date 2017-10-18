error_chain! {
    links {
        // BaseWidget(::widget::errors::Error, ::widget::errors::ErrorKind);
        // CompositeWidget(::widgets::errors::Error, ::widgets::errors::ErrorKind);
        // Input(::input::errors::Error, ::input::errors::ErrorKind);
        Resources(::resources::errors::Error, ::resources::errors::ErrorKind);
    }
}
