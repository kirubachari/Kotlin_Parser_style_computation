# Stylo Native API Integration Guide

This document provides a comprehensive technical guide for integrating with Servo's Stylo CSS engine using its native APIs to implement `getComputedStyle()` functionality.

## Executive Summary

**Stylo is not a standalone CSS library** - it's a browser engine component designed to work within the context of a complete DOM implementation. Real integration requires:

- Implementing 100+ DOM trait methods (`TElement`, `TNode`, `TDocument`)
- Managing complex lifetimes and thread safety
- Providing complete CSS parser integration
- Handling media queries, animations, and invalidation

This is why only full browser engines (Servo, Firefox) successfully integrate with Stylo.

## Core API Components

### 1. Stylist - The CSS Engine Core

```rust
use style::stylist::Stylist;
use style::media_queries::Device;
use style::context::QuirksMode;

// Create the main CSS engine
let stylist = Stylist::new(device, QuirksMode::NoQuirks);

// Add stylesheets
let guard = shared_lock.read();
stylist.append_stylesheet(stylesheet, &guard);

// Perform style resolution
let applicable_declarations = stylist.push_applicable_declarations(
    element,
    parent_bloom_filter,
    style_attribute,
    smil_override,
    animation_declarations,
    &mut applicable_declarations_cache,
    &mut matching_context,
);
```

### 2. Device - Media Environment

```rust
use style::media_queries::{Device, MediaType};
use style_traits::{CSSPixel, DevicePixel};
use euclid::{Size2D, Scale};

// Create device representing the target environment
let device = Device::new(
    MediaType::screen(),
    QuirksMode::NoQuirks,
    Size2D::<f32, CSSPixel>::new(1024.0, 768.0), // viewport
    Scale::<f32, CSSPixel, DevicePixel>::new(1.0), // device pixel ratio
    Box::new(font_metrics_provider), // font metrics
    default_computed_values, // root element styles
    PrefersColorScheme::Light, // color scheme preference
);
```

### 3. StyleContext - Resolution Context

```rust
use style::context::{SharedStyleContext, StyleContext, ThreadLocalStyleContext};
use style::shared_lock::StylesheetGuards;
use style::traversal_flags::TraversalFlags;

// Create shared context (used across threads)
let shared_context = SharedStyleContext {
    stylist: &stylist,
    visited_styles_enabled: true,
    options: StyleSystemOptions::default(),
    guards: StylesheetGuards::same(&shared_lock.read()),
    current_time_for_animations: current_time_ms(),
    traversal_flags: TraversalFlags::empty(),
    snapshot_map: &snapshot_map,
    animations: &animation_set,
    registered_speculative_painters: &painters,
};

// Create thread-local context
let mut thread_local = ThreadLocalStyleContext::new();
let mut context = StyleContext {
    shared: &shared_context,
    thread_local: &mut thread_local,
};
```

### 4. Style Resolution Pipeline

```rust
use style::style_resolver::{StyleResolverForElement, PseudoElementResolution};
use style::stylist::RuleInclusion;
use selectors::matching::IncludeStartingStyle;

// Create resolver for specific element
let mut resolver = StyleResolverForElement::new(
    element, // Must implement TElement trait
    &mut context,
    RuleInclusion::All,
    PseudoElementResolution::IfApplicable,
);

// Resolve primary style (equivalent to getComputedStyle)
let parent_style = get_parent_computed_style(element);
let resolved_styles = resolver.resolve_primary_style(
    parent_style.as_ref().map(|s| &**s),
    None, // layout_parent_style
    IncludeStartingStyle::No,
);

// Extract ComputedValues
let computed_values: Arc<ComputedValues> = resolved_styles.style.0;
```

## DOM Trait Implementation Requirements

### TElement Trait (100+ methods required)

```rust
impl TElement for BrowserElement {
    type ConcreteNode = BrowserNode;
    type TraversalChildrenIterator = BrowserChildrenIterator;

    // Basic element information
    fn local_name(&self) -> &LocalName;
    fn namespace(&self) -> &Namespace;
    fn id(&self) -> Option<&WeakAtom>;
    
    // Attribute access
    fn style_attribute(&self) -> Option<ArcBorrow<Locked<PropertyDeclarationBlock>>>;
    fn animation_rule(&self, context: &SharedStyleContext) -> Option<Arc<Locked<PropertyDeclarationBlock>>>;
    fn transition_rule(&self, context: &SharedStyleContext) -> Option<Arc<Locked<PropertyDeclarationBlock>>>;
    
    // Element state
    fn state(&self) -> ElementState;
    fn has_part_attr(&self) -> bool;
    fn exports_any_part(&self) -> bool;
    
    // Tree traversal
    fn traversal_children(&self) -> LayoutIterator<Self::TraversalChildrenIterator>;
    fn traversal_parent(&self) -> Option<Self>;
    
    // Style data management
    fn borrow_data(&self) -> Option<AtomicRef<ElementData>>;
    fn mutate_data(&self) -> Option<AtomicRefMut<ElementData>>;
    unsafe fn ensure_data(&self) -> AtomicRefMut<ElementData>;
    
    // Invalidation and dirty tracking
    fn has_dirty_descendants(&self) -> bool;
    unsafe fn set_dirty_descendants(&self);
    unsafe fn unset_dirty_descendants(&self);
    
    // Animation support
    fn may_have_animations(&self) -> bool;
    fn has_animations(&self, context: &SharedStyleContext) -> bool;
    
    // Selector matching support
    fn has_selector_flags(&self, flags: ElementSelectorFlags) -> bool;
    
    // ... 70+ more methods
}
```

### TNode Trait (30+ methods required)

```rust
impl TNode for BrowserNode {
    type ConcreteElement = BrowserElement;
    type ConcreteDocument = BrowserDocument;
    type ConcreteShadowRoot = BrowserShadowRoot;

    // Tree navigation
    fn parent_node(&self) -> Option<Self>;
    fn first_child(&self) -> Option<Self>;
    fn last_child(&self) -> Option<Self>;
    fn prev_sibling(&self) -> Option<Self>;
    fn next_sibling(&self) -> Option<Self>;
    
    // Document access
    fn owner_doc(&self) -> Self::ConcreteDocument;
    fn is_in_document(&self) -> bool;
    
    // Type conversion
    fn as_element(&self) -> Option<Self::ConcreteElement>;
    fn as_document(&self) -> Option<Self::ConcreteDocument>;
    fn as_shadow_root(&self) -> Option<Self::ConcreteShadowRoot>;
    
    // Traversal support
    fn traversal_parent(&self) -> Option<Self::ConcreteElement>;
    fn depth(&self) -> usize;
    
    // ... 15+ more methods
}
```

## ComputedValues Property Extraction

```rust
use style::properties::ComputedValues;
use style::values::computed::*;

fn extract_all_computed_properties(computed: &ComputedValues) -> HashMap<String, String> {
    let mut props = HashMap::new();
    
    // Box properties
    let box_props = computed.get_box();
    props.insert("display".to_string(), format!("{:?}", box_props.display));
    props.insert("position".to_string(), format!("{:?}", box_props.position));
    props.insert("float".to_string(), format!("{:?}", box_props.float));
    props.insert("clear".to_string(), format!("{:?}", box_props.clear));
    
    // Text properties
    let text_props = computed.get_inherited_text();
    props.insert("color".to_string(), text_props.color.to_css_string());
    props.insert("text-align".to_string(), format!("{:?}", text_props.text_align));
    props.insert("text-decoration-line".to_string(), format!("{:?}", text_props.text_decoration_line));
    
    // Font properties
    let font_props = computed.get_font();
    props.insert("font-family".to_string(), font_props.font_family.to_css_string());
    props.insert("font-size".to_string(), font_props.font_size.to_css_string());
    props.insert("font-weight".to_string(), font_props.font_weight.to_css_string());
    props.insert("font-style".to_string(), format!("{:?}", font_props.font_style));
    
    // Background properties
    let bg_props = computed.get_background();
    props.insert("background-color".to_string(), bg_props.background_color.to_css_string());
    
    // Border properties
    let border_props = computed.get_border();
    props.insert("border-top-width".to_string(), border_props.border_top_width.to_css_string());
    props.insert("border-right-width".to_string(), border_props.border_right_width.to_css_string());
    props.insert("border-bottom-width".to_string(), border_props.border_bottom_width.to_css_string());
    props.insert("border-left-width".to_string(), border_props.border_left_width.to_css_string());
    
    // Margin properties
    let margin_props = computed.get_margin();
    props.insert("margin-top".to_string(), margin_props.margin_top.to_css_string());
    props.insert("margin-right".to_string(), margin_props.margin_right.to_css_string());
    props.insert("margin-bottom".to_string(), margin_props.margin_bottom.to_css_string());
    props.insert("margin-left".to_string(), margin_props.margin_left.to_css_string());
    
    // Padding properties
    let padding_props = computed.get_padding();
    props.insert("padding-top".to_string(), padding_props.padding_top.to_css_string());
    props.insert("padding-right".to_string(), padding_props.padding_right.to_css_string());
    props.insert("padding-bottom".to_string(), padding_props.padding_bottom.to_css_string());
    props.insert("padding-left".to_string(), padding_props.padding_left.to_css_string());
    
    // Position properties
    let position_props = computed.get_position();
    props.insert("top".to_string(), position_props.top.to_css_string());
    props.insert("right".to_string(), position_props.right.to_css_string());
    props.insert("bottom".to_string(), position_props.bottom.to_css_string());
    props.insert("left".to_string(), position_props.left.to_css_string());
    props.insert("z-index".to_string(), position_props.z_index.to_css_string());
    
    // Flexbox properties
    let flex_props = computed.get_position();
    props.insert("flex-direction".to_string(), format!("{:?}", flex_props.flex_direction));
    props.insert("flex-wrap".to_string(), format!("{:?}", flex_props.flex_wrap));
    props.insert("justify-content".to_string(), format!("{:?}", flex_props.justify_content));
    props.insert("align-items".to_string(), format!("{:?}", flex_props.align_items));
    
    // Grid properties
    let grid_props = computed.get_position();
    props.insert("grid-template-columns".to_string(), grid_props.grid_template_columns.to_css_string());
    props.insert("grid-template-rows".to_string(), grid_props.grid_template_rows.to_css_string());
    
    // Transform properties
    let transform_props = computed.get_box();
    props.insert("transform".to_string(), transform_props.transform.to_css_string());
    props.insert("transform-origin".to_string(), transform_props.transform_origin.to_css_string());
    
    // Animation properties
    let animation_props = computed.get_box();
    props.insert("animation-name".to_string(), animation_props.animation_name.to_css_string());
    props.insert("animation-duration".to_string(), animation_props.animation_duration.to_css_string());
    
    // ... Extract all 500+ CSS properties
    
    props
}
```

## Performance Optimizations

### Style Sharing Cache

```rust
// Stylo includes sophisticated style sharing
let sharing_cache = context.thread_local.style_sharing_cache();
if let Some(shared_style) = sharing_cache.lookup_by_rules(element, &applicable_declarations) {
    return shared_style; // Reuse existing computed style
}

// Compute new style and add to cache
let computed_style = compute_new_style(element, &applicable_declarations);
sharing_cache.insert_if_possible(element, &computed_style, &applicable_declarations);
```

### Parallel Style Resolution

```rust
use rayon::prelude::*;
use style::parallel::traverse_subtree;

// Stylo supports parallel style computation
traverse_subtree::<BrowserElement, BrowserTraversalDriver>(
    root_element,
    &shared_context,
    &mut thread_local_contexts,
    TraversalFlags::empty(),
);
```

## Integration Complexity Analysis

### Lines of Code Required

Based on analysis of Servo and Firefox integration:

- **DOM Trait Implementation**: ~5,000-10,000 lines
- **Memory Management**: ~1,000-2,000 lines  
- **CSS Parser Integration**: ~2,000-3,000 lines
- **Media Query Support**: ~1,000-1,500 lines
- **Animation Integration**: ~3,000-5,000 lines
- **Invalidation System**: ~2,000-3,000 lines
- **Testing and Validation**: ~10,000+ lines

**Total: 25,000-35,000 lines of integration code**

### Development Time Estimate

For an experienced browser engine team:
- **Initial Integration**: 6-12 months
- **Full Feature Parity**: 12-24 months
- **Production Ready**: 24-36 months

## Conclusion

Stylo's native APIs provide unparalleled performance and correctness for CSS style computation, but require implementing a complete browser engine's DOM layer. The complexity is justified by the benefits:

- **Performance**: Parallel style resolution, sophisticated caching
- **Correctness**: Full CSS specification compliance
- **Maintainability**: Shared codebase with Firefox and Servo
- **Future-proof**: Automatic support for new CSS features

This is why Stylo integration is only practical for full browser engine implementations, not standalone applications.
