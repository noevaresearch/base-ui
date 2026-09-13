//! Primitive component utilities
//!
//! Ported from packages/react/src/primitive/index.tsx

use leptos::*;
use leptos::prelude::*;
use leptos_ui_utils::*;
use leptos_ui_internals::*;

/// Primitive component that renders any HTML element with props
///
/// This is the base component used by all menu primitives to provide the
/// as_child pattern.
pub fn Primitive(
    props: PrimitiveProps,
) -> impl IntoView {
    let PrimitiveProps { 
        element, 
        as_child, 
        children,
        .. 
    } = props;
    
    if as_child.get() {
        // Render children directly when as_child is true
        view! {
            @if let Some(children) = children {
                {children()}
            }
        }
    } else {
        // Render the specified element with all props
        let element = element.to_string();
        let mut builder = html::builder(&element);
        
        // Apply all props to the builder
        apply_primitive_props(&mut builder, &props);
        
        view! {
            @if let Some(children) = children {
                {builder.with(children)}
            } else {
                {builder}
            }
        }
    }
}

/// Primitive component properties
#[derive(Clone, Debug)]
pub struct PrimitiveProps {
    /// HTML element to render
    pub element: &'static str,
    
    /// Whether to render children directly (as_child pattern)
    pub as_child: MaybeProp<bool>,
    
    /// Children to render
    pub children: Option<Children>,
    
    /// Additional props to apply to the element
    #[allow(dead_code)]
    pub props: Vec<PrimitiveProp>,
}

impl Default for PrimitiveProps {
    fn default() -> Self {
        Self {
            element: "div",
            as_child: MaybeProp::Static(false),
            children: None,
            props: Vec::new(),
        }
    }
}

/// Primitive prop
#[derive(Clone)]
pub enum PrimitiveProp {
    /// String attribute
    Attr(&'static str, String),
    /// Boolean attribute
    BoolAttr(&'static str, bool),
    /// Event handler
    Event(&'static str, Box<dyn Fn(Event) + 'static>),
}

/// Apply primitive props to HTML builder
fn apply_primitive_props(
    builder: &mut html::AnyElementBuilder,
    props: &PrimitiveProps,
) {
    // Apply event handlers
    for prop in &props.props {
        match prop {
            PrimitiveProp::Attr(name, value) => {
                builder.attr(name, value);
            }
            PrimitiveProp::BoolAttr(name, value) => {
                builder.attr(name, value.to_string());
            }
            PrimitiveProp::Event(name, _handler) => {
                // TODO: Convert event handlers to Leptos event handlers
                // This is a simplified version - in practice we'd need
                // to map between event types
                match *name {
                    "onclick" => {
                        // TODO: Add click handler
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Create primitive props from a closure
pub fn primitive_props<F>(f: F) -> PrimitiveProps
where
    F: FnOnce(PrimitivePropsBuilder) -> PrimitiveProps,
{
    f(PrimitivePropsBuilder::new())
}

/// Builder for primitive props
pub struct PrimitivePropsBuilder {
    props: PrimitiveProps,
}

impl PrimitivePropsBuilder {
    fn new() -> Self {
        Self {
            props: PrimitiveProps::default(),
        }
    }
    
    /// Set the HTML element
    pub fn element(mut self, element: &'static str) -> Self {
        self.props.element = element;
        self
    }
    
    /// Set as_child
    pub fn as_child(mut self, as_child: bool) -> Self {
        self.props.as_child = MaybeProp::Static(as_child);
        self
    }
    
    /// Set children
    pub fn children(mut self, children: Option<Children>) -> Self {
        self.props.children = children;
        self
    }
    
    /// Add attribute
    pub fn attr(mut self, name: &'static str, value: String) -> Self {
        self.props.props.push(PrimitiveProp::Attr(name, value));
        self
    }
    
    /// Add boolean attribute
    pub fn bool_attr(mut self, name: &'static str, value: bool) -> Self {
        self.props.props.push(PrimitiveProp::BoolAttr(name, value));
        self
    }
    
    /// Add event handler
    pub fn on<F>(mut self, name: &'static str, handler: F) -> Self
    where
        F: Fn(Event) + 'static,
    {
        self.props.props.push(PrimitiveProp::Event(name, Box::new(handler)));
        self
    }
    
    /// Build the primitive props
    pub fn build(self) -> PrimitiveProps {
        self.props
    }
}