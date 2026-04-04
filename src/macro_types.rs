macro_rules! define_arg_types {
    ($($variant:ident => $type:ty),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum ArgumentType {
            $($variant),*
        }

        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        pub enum ParsedValue {
            $($variant($type)),*
        }

        impl ParsedValue {
            pub fn matches(&self, argtype: &ArgumentType) -> bool {
                matches!(
                    (&self, argtype),
                    $((ParsedValue::$variant(_), ArgumentType::$variant))|*
                )
            }
        }
    };
}

define_arg_types! {
    Integer => i64,
    Text    => String,
    Boolean => bool,
}
