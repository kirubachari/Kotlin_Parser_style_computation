# Servo-Based Computed Style API Implementation Plan

## Executive Summary

**VERDICT: Using Servo to access Stylo's computed styles is not only viable but the optimal solution!**

After examining Servo's codebase, I found that Servo already implements the exact functionality we need with complete Stylo integration. This approach leverages Servo's existing `getComputedStyle()` implementation which directly uses Stylo's native APIs.

## How Servo's getComputedStyle() Uses Stylo

### 1. API Call Flow

```
JavaScript: window.getComputedStyle(element)
    ↓
Servo DOM: Window::GetComputedStyle() 
    ↓
Layout Query: resolved_style_query()
    ↓
Stylo Integration: process_resolved_style_request()
    ↓
Stylo APIs: resolve_style() with SharedStyleContext
    ↓
Result: ComputedValues → CSS string
```

### 2. Core Implementation (from Servo's codebase)

**Window::GetComputedStyle()** (`servo/components/script/dom/window.rs:1534`)
```rust
fn GetComputedStyle(&self, element: &Element, pseudo: Option<DOMString>) -> DomRoot<CSSStyleDeclaration> {
    // Creates CSSStyleDeclaration that calls resolved_style_query for each property
    CSSStyleDeclaration::new(self, CSSStyleOwner::Element(Dom::from_ref(element)), pseudo, CSSModificationAccess::Readonly, CanGc::note())
}
```

**resolved_style_query()** (`servo/components/script/dom/window.rs:2578`)
```rust
pub(crate) fn resolved_style_query(&self, element: TrustedNodeAddress, pseudo: Option<PseudoElement>, property: PropertyId) -> DOMString {
    self.layout_reflow(QueryMsg::ResolvedStyleQuery);
    let document = self.Document();
    let animations = document.animations().sets.clone();
    DOMString::from(self.layout.borrow().query_resolved_style(element, pseudo, property, animations, document.current_animation_timeline_value()))
}
```

**query_resolved_style()** (`servo/components/layout/layout_impl.rs:339`)
```rust
fn query_resolved_style(&self, node: TrustedNodeAddress, pseudo: Option<PseudoElement>, property_id: PropertyId, animations: DocumentAnimationSet, animation_timeline_value: f64) -> String {
    let node = unsafe { ServoLayoutNode::new(&node) };
    let document = node.owner_doc();
    let document_shared_lock = document.style_shared_lock();
    let guards = StylesheetGuards {
        author: &document_shared_lock.read(),
        ua_or_user: &UA_STYLESHEETS.shared_lock.read(),
    };
    let snapshot_map = SnapshotMap::new();

    let shared_style_context = self.build_shared_style_context(guards, &snapshot_map, animation_timeline_value, &animations, TraversalFlags::empty());

    process_resolved_style_request(&shared_style_context, node, &pseudo, &property_id)
}
```

**process_resolved_style_request()** (`servo/components/layout/query.rs:158`) - **THE CORE STYLO INTEGRATION**
```rust
pub fn process_resolved_style_request(context: &SharedStyleContext, node: ServoLayoutNode<'_>, pseudo: &Option<PseudoElement>, property: &PropertyId) -> String {
    if !node.as_element().unwrap().has_data() {
        return process_resolved_style_request_for_unstyled_node(context, node, pseudo, property);
    }

    // Get the layout element with computed styles
    let layout_element = node.to_threadsafe().as_element().unwrap();
    let layout_element = match pseudo {
        Some(pseudo_element_type) => layout_element.with_pseudo(*pseudo_element_type).unwrap_or(layout_element),
        None => layout_element,
    };

    // Extract computed value using Stylo's ComputedValues
    let computed_values = layout_element.style_data();
    let style = &*computed_values.styles.primary();
    
    // Convert Stylo's computed value to CSS string
    match *property {
        PropertyId::NonCustom(id) => match id.longhand_or_shorthand() {
            Ok(longhand_id) => style.computed_value_to_string(PropertyDeclarationId::Longhand(longhand_id)),
            Err(shorthand_id) => shorthand_to_css_string(shorthand_id, style),
        },
        PropertyId::Custom(ref name) => style.computed_value_to_string(PropertyDeclarationId::Custom(name)),
    }
}
```

**For unstyled elements** (`servo/components/layout/query.rs:415`):
```rust
pub fn process_resolved_style_request_for_unstyled_node(context: &SharedStyleContext, node: ServoLayoutNode<'_>, pseudo: &Option<PseudoElement>, property: &PropertyId) -> String {
    let mut tlc = ThreadLocalStyleContext::new();
    let mut context = StyleContext { shared: context, thread_local: &mut tlc };
    
    let element = node.as_element().unwrap();
    // **DIRECT STYLO API CALL**
    let styles = resolve_style(&mut context, element, RuleInclusion::All, pseudo.as_ref(), None);
    let style = styles.primary();
    
    // Extract property value from Stylo's ComputedValues
    style.computed_value_to_string(PropertyDeclarationId::Longhand(longhand_id))
}
```

## Implementation Approaches

### Approach 1: Servo Embedding API (Recommended)

Create a minimal API that embeds Servo's layout engine:

```rust
use servo_components::layout::LayoutImpl;
use servo_components::script::dom::{Element, Window};
use style::properties::PropertyId;

pub struct ServoStyleEngine {
    layout: LayoutImpl,
    document: Document,
}

impl ServoStyleEngine {
    pub fn new() -> Result<Self, ServoError> {
        // Initialize minimal Servo components
        let layout = LayoutImpl::new(/* minimal config */);
        let document = Document::new(/* basic HTML document */);
        Ok(ServoStyleEngine { layout, document })
    }
    
    pub fn add_stylesheet(&mut self, css: &str) -> Result<(), ServoError> {
        // Add CSS to document stylesheets
        self.document.add_stylesheet(css)
    }
    
    pub fn set_html(&mut self, html: &str) -> Result<(), ServoError> {
        // Parse HTML and build DOM
        self.document.set_inner_html(html)
    }
    
    pub fn get_computed_style(&self, selector: &str, property: &str) -> Result<String, ServoError> {
        // 1. Find element by selector
        let element = self.document.query_selector(selector)?;
        
        // 2. Parse property ID
        let property_id = PropertyId::parse_enabled_for_all_content(property)?;
        
        // 3. Call Servo's native implementation
        let node_address = element.to_trusted_node_address();
        let computed_value = self.layout.query_resolved_style(
            node_address,
            None, // pseudo
            property_id,
            DocumentAnimationSet::default(),
            0.0, // animation_timeline_value
        );
        
        Ok(computed_value)
    }
    
    pub fn get_all_computed_styles(&self, selector: &str) -> Result<HashMap<String, String>, ServoError> {
        let element = self.document.query_selector(selector)?;
        let mut styles = HashMap::new();
        
        // Iterate through all CSS properties
        for property_name in ALL_CSS_PROPERTIES {
            let value = self.get_computed_style(selector, property_name)?;
            styles.insert(property_name.to_string(), value);
        }
        
        Ok(styles)
    }
}
```

### Approach 2: Servo Process API (Alternative)

Use Servo as a subprocess with custom protocol:

```rust
use std::process::{Command, Stdio};
use serde_json::{json, Value};

pub struct ServoStyleProcess {
    process: std::process::Child,
}

impl ServoStyleProcess {
    pub fn new() -> Result<Self, ServoError> {
        let process = Command::new("servo")
            .arg("--headless")
            .arg("--style-query-mode") // Custom flag we'd add to Servo
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
            
        Ok(ServoStyleProcess { process })
    }
    
    pub fn query_style(&mut self, html: &str, css: &str, selector: &str, property: &str) -> Result<String, ServoError> {
        let request = json!({
            "html": html,
            "css": css,
            "selector": selector,
            "property": property
        });
        
        // Send request to Servo process
        serde_json::to_writer(self.process.stdin.as_mut().unwrap(), &request)?;
        
        // Read response
        let response: Value = serde_json::from_reader(self.process.stdout.as_mut().unwrap())?;
        Ok(response["computed_value"].as_str().unwrap().to_string())
    }
}
```

## Advantages of Servo Approach

### ✅ **Uses Actual Stylo APIs**
- `resolve_style()` - Stylo's core style resolution function
- `SharedStyleContext` - Stylo's style computation context
- `ComputedValues` - Stylo's computed property values
- `PropertyDeclarationId` - Stylo's property identification system

### ✅ **Complete CSS Support**
- All CSS properties supported by Stylo/Firefox
- Proper cascade resolution and inheritance
- Media query evaluation
- Pseudo-element support
- Animation and transition handling

### ✅ **Production Quality**
- Battle-tested in Servo browser
- Same code path as Firefox's style system
- Comprehensive error handling
- Performance optimizations (style sharing, caching)

### ✅ **Maintainable**
- No custom DOM trait implementations needed
- Leverages existing Servo infrastructure
- Automatic updates with Servo/Stylo improvements

## Implementation Complexity

### Approach 1 (Embedding): Medium Complexity
- **Estimated effort**: 2-4 weeks
- **Lines of code**: 1,000-2,000 lines
- **Dependencies**: Servo layout and script components
- **Challenges**: Minimal Servo initialization, DOM manipulation APIs

### Approach 2 (Process): Low Complexity  
- **Estimated effort**: 1-2 weeks
- **Lines of code**: 500-1,000 lines
- **Dependencies**: Servo binary with custom flags
- **Challenges**: Process communication, Servo modifications

## Recommended Implementation

**Start with Approach 2 (Process API)** for proof of concept:

1. **Modify Servo** to accept style queries via stdin/stdout
2. **Create simple Rust wrapper** that communicates with Servo process
3. **Validate that it works** with complex CSS and HTML
4. **Optimize for performance** if needed

**Upgrade to Approach 1 (Embedding)** for production:

1. **Extract minimal Servo components** needed for style resolution
2. **Create clean API** that hides Servo complexity
3. **Add comprehensive error handling** and validation
4. **Optimize memory usage** and startup time

## Next Steps

1. **Examine Servo's build system** to understand component dependencies
2. **Create minimal Servo modification** for style query mode
3. **Implement proof-of-concept** with simple HTML/CSS examples
4. **Test with complex scenarios** (animations, media queries, pseudo-elements)
5. **Benchmark performance** against other solutions

This approach gives you **genuine Stylo computed styles** with **production-quality CSS support** while avoiding the complexity of direct Stylo integration!
