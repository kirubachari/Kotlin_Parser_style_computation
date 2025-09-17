//! Implementation of Stylo's DOM traits for our simplified DOM elements.
//!
//! This module provides implementations of TElement, TNode, and TDocument traits
//! that are required by Stylo's style resolution system.

use std::collections::HashMap;
use std::sync::Arc;
use atomic_refcell::{AtomicRef, AtomicRefMut, AtomicRefCell};

use style::context::QuirksMode;
use style::data::ElementData;
use style::dom::{TDocument, TElement, TNode, TShadowRoot, NodeInfo, LayoutIterator};
use style::media_queries::Device;
use style::properties::{ComputedValues, PropertyDeclarationBlock, AnimationDeclarations};
use style::selector_parser::{SelectorImpl, PseudoElement, AttrValue, Lang, RestyleDamage};
use style::shared_lock::{SharedRwLock, Locked};
use style::stylist::CascadeData;
use style::values::computed::Display;
use style::values::AtomIdent;
use style::{LocalName, WeakAtom};
use servo_arc::Arc as ServoArc;
use selectors::matching::{ElementSelectorFlags, VisitedHandlingMode};
use selectors::sink::Push;
use selectors::{Element as SelectorsElement, OpaqueElement};
use style_traits::dom::OpaqueNode;

/// A simple document implementation for Stylo
#[derive(Debug, Clone, Copy)]
pub struct StyloDocument {
    shared_lock: *const SharedRwLock,
    quirks_mode: QuirksMode,
}

unsafe impl Send for StyloDocument {}
unsafe impl Sync for StyloDocument {}

impl StyloDocument {
    pub fn new(shared_lock: &SharedRwLock, quirks_mode: QuirksMode) -> Self {
        Self {
            shared_lock: shared_lock as *const SharedRwLock,
            quirks_mode,
        }
    }
}

/// A simple node implementation for Stylo
#[derive(Debug, Clone)]
pub struct StyloNode {
    pub element: Option<StyloElement>,
    pub document: StyloDocument,
}

impl Copy for StyloNode {}

impl StyloNode {
    pub fn new_element(element: StyloElement, document: StyloDocument) -> Self {
        Self {
            element: Some(element),
            document,
        }
    }
    
    pub fn new_document(document: StyloDocument) -> Self {
        Self {
            element: None,
            document,
        }
    }
}

/// A simple element implementation for Stylo
#[derive(Debug, Clone)]
pub struct StyloElement {
    pub tag_name: LocalName,
    pub attributes: HashMap<LocalName, AttrValue>,
    pub parent: Option<Arc<StyloElement>>,
    pub children: Vec<Arc<StyloElement>>,
    pub data: AtomicRefCell<Option<ElementData>>,
    pub id: Option<WeakAtom>,
    pub classes: Vec<AtomIdent>,
}

impl Copy for StyloElement {}

impl StyloElement {
    pub fn new(tag_name: &str) -> Self {
        Self {
            tag_name: LocalName::from(tag_name),
            attributes: HashMap::new(),
            parent: None,
            children: Vec::new(),
            data: AtomicRefCell::new(None),
            id: None,
            classes: Vec::new(),
        }
    }
    
    pub fn with_attribute(mut self, name: &str, value: &str) -> Self {
        self.attributes.insert(LocalName::from(name), AttrValue::from(value));
        
        // Handle special attributes
        if name == "id" {
            self.id = Some(WeakAtom::from(value));
        } else if name == "class" {
            self.classes = value.split_whitespace()
                .map(|c| AtomIdent::from(c))
                .collect();
        }
        
        self
    }
}

// Implement required traits for StyloDocument
impl TDocument for StyloDocument {
    type ConcreteNode = StyloNode;

    fn as_node(&self) -> Self::ConcreteNode {
        StyloNode::new_document(*self)
    }

    fn is_html_document(&self) -> bool {
        true
    }

    fn quirks_mode(&self) -> QuirksMode {
        self.quirks_mode
    }

    fn shared_lock(&self) -> &SharedRwLock {
        unsafe { &*self.shared_lock }
    }
}

// Implement NodeInfo for StyloNode
impl NodeInfo for StyloNode {
    fn is_element(&self) -> bool {
        self.element.is_some()
    }

    fn is_text_node(&self) -> bool {
        false
    }
}

// Implement TNode for StyloNode
impl TNode for StyloNode {
    type ConcreteElement = StyloElement;
    type ConcreteDocument = StyloDocument;
    type ConcreteShadowRoot = StyloShadowRoot;

    fn parent_node(&self) -> Option<Self> {
        self.element?.parent.as_ref().map(|p| {
            StyloNode::new_element(**p, self.document)
        })
    }

    fn first_child(&self) -> Option<Self> {
        self.element?.children.first().map(|c| {
            StyloNode::new_element(**c, self.document)
        })
    }

    fn last_child(&self) -> Option<Self> {
        self.element?.children.last().map(|c| {
            StyloNode::new_element(**c, self.document)
        })
    }

    fn prev_sibling(&self) -> Option<Self> {
        // Simplified implementation - would need proper sibling tracking
        None
    }

    fn next_sibling(&self) -> Option<Self> {
        // Simplified implementation - would need proper sibling tracking
        None
    }

    fn owner_doc(&self) -> Self::ConcreteDocument {
        self.document
    }

    fn is_in_document(&self) -> bool {
        true
    }

    fn traversal_parent(&self) -> Option<Self::ConcreteElement> {
        self.element?.parent.as_ref().map(|p| **p)
    }

    fn opaque(&self) -> OpaqueNode {
        OpaqueNode::from_ptr(self as *const _ as *const ())
    }

    fn debug_id(self) -> usize {
        self as *const _ as usize
    }

    fn as_element(&self) -> Option<Self::ConcreteElement> {
        self.element
    }

    fn as_document(&self) -> Option<Self::ConcreteDocument> {
        if self.element.is_none() {
            Some(self.document)
        } else {
            None
        }
    }

    fn as_shadow_root(&self) -> Option<Self::ConcreteShadowRoot> {
        None
    }
}

// Placeholder shadow root implementation
#[derive(Debug, Clone, Copy)]
pub struct StyloShadowRoot;

impl TShadowRoot for StyloShadowRoot {
    type ConcreteNode = StyloNode;

    fn as_node(&self) -> Self::ConcreteNode {
        unimplemented!("Shadow roots not implemented in this example")
    }

    fn host(&self) -> StyloElement {
        unimplemented!("Shadow roots not implemented in this example")
    }

    fn style_data<'a>(&self) -> Option<&'a CascadeData>
    where
        Self: 'a,
    {
        None
    }
}

impl PartialEq for StyloNode {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl PartialEq for StyloElement {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl std::hash::Hash for StyloElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self as *const Self).hash(state);
    }
}

// Implement SelectorsElement for StyloElement (required by TElement)
impl SelectorsElement for StyloElement {
    type Impl = SelectorImpl;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::from_ptr(self as *const _ as *const ())
    }

    fn parent_element(&self) -> Option<Self> {
        self.parent.as_ref().map(|p| **p)
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        None // Simplified
    }

    fn next_sibling_element(&self) -> Option<Self> {
        None // Simplified
    }

    fn is_html_element_in_html_document(&self) -> bool {
        true
    }

    fn has_local_name(&self, local_name: &LocalName) -> bool {
        &self.tag_name == local_name
    }

    fn has_namespace(&self, _ns: &style::Namespace) -> bool {
        true // Simplified - assume HTML namespace
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.tag_name == other.tag_name
    }

    fn attr_matches(
        &self,
        _ns: &selectors::attr::NamespaceConstraint<&style::Namespace>,
        local_name: &LocalName,
        operation: &selectors::attr::AttrSelectorOperation<&AttrValue>,
    ) -> bool {
        if let Some(attr_value) = self.attributes.get(local_name) {
            operation.matches(attr_value)
        } else {
            false
        }
    }

    fn match_non_ts_pseudo_class(
        &self,
        _pc: &style::selector_parser::NonTSPseudoClass,
        _context: &mut selectors::matching::MatchingContext<Self::Impl>,
    ) -> bool {
        false // Simplified
    }

    fn match_pseudo_element(
        &self,
        _pe: &PseudoElement,
        _context: &mut selectors::matching::MatchingContext<Self::Impl>,
    ) -> bool {
        false // Simplified
    }

    fn is_link(&self) -> bool {
        self.tag_name.as_ref() == "a"
    }

    fn is_html_slot_element(&self) -> bool {
        false
    }

    fn has_id(&self, id: &AtomIdent, _case_sensitivity: selectors::attr::CaseSensitivity) -> bool {
        self.id.as_ref().map_or(false, |self_id| {
            self_id.as_ref() == id.as_ref()
        })
    }

    fn has_class(&self, name: &AtomIdent, _case_sensitivity: selectors::attr::CaseSensitivity) -> bool {
        self.classes.iter().any(|class| class == name)
    }

    fn imported_part(&self, _name: &AtomIdent) -> Option<AtomIdent> {
        None
    }

    fn is_part(&self, _name: &AtomIdent) -> bool {
        false
    }

    fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

// Implement TElement for StyloElement
impl TElement for StyloElement {
    type ConcreteNode = StyloNode;
    type TraversalChildrenIterator = std::vec::IntoIter<StyloNode>;

    fn as_node(&self) -> Self::ConcreteNode {
        // We need a document reference - this is a simplified approach
        let shared_lock = SharedRwLock::new();
        let document = StyloDocument::new(&shared_lock, QuirksMode::NoQuirks);
        StyloNode::new_element(*self, document)
    }

    fn traversal_children(&self) -> LayoutIterator<Self::TraversalChildrenIterator> {
        let shared_lock = SharedRwLock::new();
        let document = StyloDocument::new(&shared_lock, QuirksMode::NoQuirks);

        let children: Vec<StyloNode> = self.children.iter()
            .map(|child| StyloNode::new_element(**child, document))
            .collect();

        LayoutIterator(children.into_iter())
    }

    fn is_html_element(&self) -> bool {
        true // Simplified - assume all elements are HTML
    }

    fn is_mathml_element(&self) -> bool {
        false
    }

    fn is_svg_element(&self) -> bool {
        false
    }

    fn style_attribute(&self) -> Option<servo_arc::ArcBorrow<Locked<PropertyDeclarationBlock>>> {
        None // Simplified - no style attributes
    }

    fn animation_rule(
        &self,
        _context: &style::context::SharedStyleContext,
    ) -> Option<ServoArc<Locked<PropertyDeclarationBlock>>> {
        None
    }

    fn transition_rule(
        &self,
        _context: &style::context::SharedStyleContext,
    ) -> Option<ServoArc<Locked<PropertyDeclarationBlock>>> {
        None
    }

    fn state(&self) -> style::dom::ElementState {
        style::dom::ElementState::empty()
    }

    fn has_part_attr(&self) -> bool {
        false
    }

    fn exports_any_part(&self) -> bool {
        false
    }

    fn id(&self) -> Option<&WeakAtom> {
        self.id.as_ref()
    }

    fn each_class<F>(&self, mut callback: F)
    where
        F: FnMut(&AtomIdent),
    {
        for class in &self.classes {
            callback(class);
        }
    }

    fn each_custom_state<F>(&self, _callback: F)
    where
        F: FnMut(&AtomIdent),
    {
        // No custom states in this simplified implementation
    }

    fn each_attr_name<F>(&self, mut callback: F)
    where
        F: FnMut(&LocalName),
    {
        for name in self.attributes.keys() {
            callback(name);
        }
    }

    fn has_dirty_descendants(&self) -> bool {
        false // Simplified
    }

    fn has_snapshot(&self) -> bool {
        false
    }

    fn handled_snapshot(&self) -> bool {
        true
    }

    unsafe fn set_handled_snapshot(&self) {
        // No-op in simplified implementation
    }

    unsafe fn set_dirty_descendants(&self) {
        // No-op in simplified implementation
    }

    unsafe fn unset_dirty_descendants(&self) {
        // No-op in simplified implementation
    }

    fn store_children_to_process(&self, _n: isize) {
        // No-op in simplified implementation
    }

    fn did_process_child(&self) -> isize {
        0 // Simplified
    }

    unsafe fn ensure_data(&self) -> AtomicRefMut<ElementData> {
        let mut data = self.data.borrow_mut();
        if data.is_none() {
            *data = Some(ElementData::new(None));
        }
        AtomicRefMut::map(data, |d| d.as_mut().unwrap())
    }

    unsafe fn clear_data(&self) {
        *self.data.borrow_mut() = None;
    }

    fn has_data(&self) -> bool {
        self.data.borrow().is_some()
    }

    fn borrow_data(&self) -> Option<AtomicRef<ElementData>> {
        if self.has_data() {
            Some(AtomicRef::map(self.data.borrow(), |d| d.as_ref().unwrap()))
        } else {
            None
        }
    }

    fn mutate_data(&self) -> Option<AtomicRefMut<ElementData>> {
        if self.has_data() {
            Some(AtomicRefMut::map(self.data.borrow_mut(), |d| d.as_mut().unwrap()))
        } else {
            None
        }
    }

    fn skip_item_display_fixup(&self) -> bool {
        false
    }

    fn may_have_animations(&self) -> bool {
        false
    }

    fn has_animations(&self, _context: &style::context::SharedStyleContext) -> bool {
        false
    }

    fn has_css_animations(
        &self,
        _context: &style::context::SharedStyleContext,
        _pseudo_element: Option<PseudoElement>,
    ) -> bool {
        false
    }

    fn has_css_transitions(
        &self,
        _context: &style::context::SharedStyleContext,
        _pseudo_element: Option<PseudoElement>,
    ) -> bool {
        false
    }

    fn shadow_root(&self) -> Option<StyloShadowRoot> {
        None
    }

    fn containing_shadow(&self) -> Option<StyloShadowRoot> {
        None
    }

    fn lang_attr(&self) -> Option<AttrValue> {
        self.attributes.get(&LocalName::from("lang")).cloned()
    }

    fn match_element_lang(
        &self,
        _override_lang: Option<Option<AttrValue>>,
        _value: &Lang,
    ) -> bool {
        false // Simplified
    }

    fn is_html_document_body_element(&self) -> bool {
        self.tag_name.as_ref() == "body"
    }

    fn synthesize_presentational_hints_for_legacy_attributes<V>(
        &self,
        _visited_handling: VisitedHandlingMode,
        _hints: &mut V,
    ) where
        V: Push<style::applicable_declarations::ApplicableDeclarationBlock>,
    {
        // No presentational hints in this simplified implementation
    }

    fn local_name(&self) -> &LocalName {
        &self.tag_name
    }

    fn namespace(&self) -> &style::Namespace {
        // Return HTML namespace - this is a simplified approach
        unsafe { std::mem::transmute("http://www.w3.org/1999/xhtml") }
    }

    fn query_container_size(
        &self,
        _display: &Display,
    ) -> euclid::default::Size2D<Option<app_units::Au>> {
        euclid::default::Size2D::new(None, None)
    }

    fn has_selector_flags(&self, _flags: ElementSelectorFlags) -> bool {
        false
    }

    fn relative_selector_search_direction(&self) -> ElementSelectorFlags {
        ElementSelectorFlags::empty()
    }
}
