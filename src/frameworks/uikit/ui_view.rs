/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIView`.

pub mod ui_alert_view;
pub mod ui_control;
pub mod ui_image_view;
pub mod ui_label;
pub mod ui_window;

use super::ui_graphics::{UIGraphicsPopContext, UIGraphicsPushContext};
use crate::frameworks::core_graphics::cg_context::{CGContextClearRect, CGContextRef};
use crate::frameworks::core_graphics::{CGFloat, CGPoint, CGRect};
use crate::frameworks::foundation::ns_string::get_static_str;
use crate::frameworks::foundation::NSUInteger;
use crate::objc::{
    id, msg, nil, objc_classes, release, retain, Class, ClassExports, HostObject, NSZonePtr,
};

#[derive(Default)]
pub struct State {
    /// List of views for internal purposes. Non-retaining!
    pub(super) views: Vec<id>,
    pub ui_window: ui_window::State,
}

pub(super) struct UIViewHostObject {
    /// CALayer or subclass.
    layer: id,
    /// Subviews in back-to-front order. These are strong references.
    subviews: Vec<id>,
    /// The superview. This is a weak reference.
    superview: id,
    clears_context_before_drawing: bool,
    user_interaction_enabled: bool,
}
impl HostObject for UIViewHostObject {}
impl Default for UIViewHostObject {
    fn default() -> UIViewHostObject {
        // The Default trait is implemented so subclasses will get the same
        // defaults.
        UIViewHostObject {
            layer: nil,
            subviews: Vec::new(),
            superview: nil,
            clears_context_before_drawing: true,
            user_interaction_enabled: true,
        }
    }
}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIView: UIResponder

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::<UIViewHostObject>::default();
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (Class)layerClass {
    env.objc.get_known_class("CALayer", &mut env.mem)
}

// TODO: accessors etc

- (id)init {
    let view_class: Class = msg![env; this class];
    let layer_class: Class = msg![env; view_class layerClass];
    let layer: id = msg![env; layer_class layer];

    // CALayer is not opaque by default, but UIView is
    () = msg![env; layer setDelegate:this];
    () = msg![env; layer setOpaque:true];

    env.objc.borrow_mut::<UIViewHostObject>(this).layer = layer;

    env.framework_state.uikit.ui_view.views.push(this);

    this
}

- (id)initWithFrame:(CGRect)frame {
    let this: id = msg![env; this init];

    () = msg![env; this setFrame:frame];

    log_dbg!(
        "[(UIView*){:?} initWithFrame:{:?}] => bounds {:?}, center {:?}",
        this,
        frame,
        { let bounds: CGRect = msg![env; this bounds]; bounds },
        { let center: CGPoint = msg![env; this center]; center },
    );

    this
}

// NSCoding implementation
- (id)initWithCoder:(id)coder {
    let this: id = msg![env; this init];

    // TODO: decode the various other UIView properties

    let key_ns_string = get_static_str(env, "UIBounds");
    let bounds: CGRect = msg![env; coder decodeCGRectForKey:key_ns_string];

    let key_ns_string = get_static_str(env, "UICenter");
    let center: CGPoint = msg![env; coder decodeCGPointForKey:key_ns_string];

    let key_ns_string = get_static_str(env, "UIHidden");
    let hidden: bool = msg![env; coder decodeBoolForKey:key_ns_string];

    let key_ns_string = get_static_str(env, "UIOpaque");
    let opaque: bool = msg![env; coder decodeBoolForKey:key_ns_string];

    let key_ns_string = get_static_str(env, "UISubviews");
    let subviews: id = msg![env; coder decodeObjectForKey:key_ns_string];
    let subview_count: NSUInteger = msg![env; subviews count];

    log_dbg!(
        "[(UIView*){:?} initWithCoder:{:?}] => bounds {}, center {}, hidden {}, opaque {}, {} subviews",
        this,
        coder,
        bounds,
        center,
        hidden,
        opaque,
        subview_count,
    );

    () = msg![env; this setBounds:bounds];
    () = msg![env; this setCenter:center];
    () = msg![env; this setHidden:hidden];
    () = msg![env; this setOpaque:opaque];

    for i in 0..subview_count {
        let subview: id = msg![env; subviews objectAtIndex:i];
        () = msg![env; this addSubview:subview];
    }

    this
}

- (bool)isUserInteractionEnabled {
    env.objc.borrow::<UIViewHostObject>(this).user_interaction_enabled
}
- (())setUserInteractionEnabled:(bool)enabled {
    env.objc.borrow_mut::<UIViewHostObject>(this).user_interaction_enabled = enabled;
}

// TODO: setMultipleTouchEnabled
- (())setMultipleTouchEnabled:(bool)_enabled {
    // TODO: enable multitouch
}

- (())layoutSubviews {
    // On iOS 5.1 and earlier, the default implementation of this method does nothing.
}

- (id)superview {
    env.objc.borrow::<UIViewHostObject>(this).superview
}
// TODO: subviews accessor

- (())addSubview:(id)view {
    log_dbg!("[(UIView*){:?} addSubview:{:?}] => ()", this, view);
    if view != nil {
        if env.objc.borrow::<UIViewHostObject>(view).superview == this {
            () = msg![env; this bringSubviewToFront:view];
        } else {
            retain(env, view);
            () = msg![env; view removeFromSuperview];
            let subview_obj = env.objc.borrow_mut::<UIViewHostObject>(view);
            subview_obj.superview = this;
            let subview_layer = subview_obj.layer;
            let this_obj = env.objc.borrow_mut::<UIViewHostObject>(this);
            this_obj.subviews.push(view);
            let this_layer = this_obj.layer;
            () = msg![env; this_layer addSublayer:subview_layer];
        }
    }
}

- (())bringSubviewToFront:(id)subview {
    let &mut UIViewHostObject {
        ref mut subviews,
        layer,
        ..
    } = env.objc.borrow_mut(this);

    let idx = subviews.iter().position(|&subview2| subview2 == subview).unwrap();
    let subview2 = subviews.remove(idx);
    assert!(subview2 == subview);
    subviews.push(subview);

    let subview_layer = env.objc.borrow::<UIViewHostObject>(subview).layer;
    () = msg![env; subview_layer removeFromSuperlayer];
    () = msg![env; layer addSublayer:subview_layer];
}

- (())removeFromSuperview {
    let &mut UIViewHostObject {
        ref mut superview,
        layer: this_layer,
        ..
    } = env.objc.borrow_mut(this);
    let superview = std::mem::take(superview);
    if superview == nil {
        return;
    }
    () = msg![env; this_layer removeFromSuperlayer];

    let UIViewHostObject { ref mut subviews, .. } = env.objc.borrow_mut(superview);
    let idx = subviews.iter().position(|&subview| subview == this).unwrap();
    let subview = subviews.remove(idx);
    assert!(subview == this);
    release(env, this);
}

- (())dealloc {
    let UIViewHostObject {
        layer,
        superview,
        subviews,
        clears_context_before_drawing: _,
        user_interaction_enabled: _,
    } = std::mem::take(env.objc.borrow_mut(this));

    release(env, layer);
    assert!(superview == nil);
    for subview in subviews {
        env.objc.borrow_mut::<UIViewHostObject>(subview).superview = nil;
        release(env, subview);
    }

    env.framework_state.uikit.ui_view.views.swap_remove(
        env.framework_state.uikit.ui_view.views.iter().position(|&v| v == this).unwrap()
    );

    env.objc.dealloc_object(this, &mut env.mem);
}

- (id)layer {
    env.objc.borrow_mut::<UIViewHostObject>(this).layer
}

- (bool)isHidden {
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer isHidden]
}
- (())setHidden:(bool)hidden {
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer setHidden:hidden]
}

- (bool)isOpaque {
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer isOpaque]
}
- (())setOpaque:(bool)opaque {
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer setOpaque:opaque]
}

- (CGFloat)alpha {
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer opacity]
}
- (())setAlpha:(CGFloat)alpha {
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer setOpacity:alpha]
}

// FIXME: CALayer's backgroundColor should be a CGColorRef, which is supposedly
// a separate type from UIColor. For now we have not implemented it and treat
// them as the same type (and it seems like UIKit itself maybe did this once),
// but eventually we'll have to do this properly.
- (id)backgroundColor {
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer backgroundColor]
}
- (())setBackgroundColor:(id)color { // UIColor*
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer setBackgroundColor:color]
}

// TODO: support setNeedsDisplayInRect:
- (())setNeedsDisplay {
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer setNeedsDisplay]
}

- (CGRect)bounds {
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer bounds]
}
- (())setBounds:(CGRect)bounds {
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer setBounds:bounds]
}
- (CGPoint)center {
    // FIXME: what happens if [layer anchorPoint] isn't (0.5, 0.5)?
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer position]
}
- (())setCenter:(CGRect)center {
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer setPosition:center]
}
- (CGRect)frame {
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer frame]
}
- (())setFrame:(CGRect)frame {
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer setFrame:frame]
}

// Is this correct? The panic message complained about not finding EAGLView setContentMode
- (())setContentMode:(id)content_mode {
    log!["TODO: [UIView {:?} setContentMode:{:?} => ()", this, content_mode ];
}

- (bool)clearsContextBeforeDrawing {
    env.objc.borrow::<UIViewHostObject>(this).clears_context_before_drawing
}
- (())setClearsContextBeforeDrawing:(bool)v {
    env.objc.borrow_mut::<UIViewHostObject>(this).clears_context_before_drawing = v;
}

// Drawing stuff that views should override
- (())drawRect:(CGRect)_rect {
    // default implementation does nothing
}

// CALayerDelegate implementation
- (())drawLayer:(id)layer // CALayer*
      inContext:(CGContextRef)context {
    let mut bounds: CGRect = msg![env; layer bounds];
    bounds.origin = CGPoint { x: 0.0, y: 0.0 }; // FIXME: not tested
    if env.objc.borrow::<UIViewHostObject>(this).clears_context_before_drawing {
        CGContextClearRect(env, context, bounds);
    }
    UIGraphicsPushContext(env, context);
    () = msg![env; this drawRect:bounds];
    UIGraphicsPopContext(env);
}

// Event handling

- (id)pointInside:(CGPoint)point
        withEvent:(id)_event { // UIEvent* (possibly nil)
    let layer = env.objc.borrow::<UIViewHostObject>(this).layer;
    msg![env; layer containsPoint:point]
}

- (id)hitTest:(CGPoint)point
    withEvent:(id)event { // UIEvent* (possibly nil)
    if !msg![env; this pointInside:point withEvent:event] {
        return nil;
    }
    // TODO: avoid copy somehow?
    let subviews = env.objc.borrow::<UIViewHostObject>(this).subviews.clone();
    for subview in subviews.into_iter().rev() { // later views are on top
        let hidden: bool = msg![env; subview isHidden];
        let alpha: CGFloat = msg![env; subview alpha];
        let interactible: bool = msg![env; this isUserInteractionEnabled];
        if hidden || alpha < 0.01 || !interactible {
           continue;
        }
        let frame: CGRect = msg![env; subview frame];
        let bounds: CGRect = msg![env; subview bounds];
        let point = CGPoint {
            x: point.x - frame.origin.x + bounds.origin.x,
            y: point.y - frame.origin.y + bounds.origin.y,
        };
        let subview: id = msg![env; subview hitTest:point withEvent:event];
        if subview != nil {
            return subview;
        }
    }
    this
}

@end

};
