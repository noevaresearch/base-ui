//! Menu constants
//! 
//! Ported from packages/react/src/menu/constants.ts

/// Menu component constants
pub mod constants {
    /// Menu event names
    pub const MENU_EVENTS: [&str; 7] = [
        "keydown",
        "mousedown", 
        "mouseup",
        "mouseenter",
        "mouseleave",
        "focusin",
        "focusout",
    ];
    
    /// Menu default styles
    pub const MENU_DEFAULT_STYLES: &str = r#"
        .menu {
            position: relative;
        }
        
        .menu-trigger {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            gap: 4px;
            font-weight: 500;
            border-radius: 6px;
            padding: 0 12px;
            height: 36px;
            cursor: pointer;
            transition: background-color 0.2s;
        }
        
        .menu-trigger:hover {
            background-color: rgba(0, 0, 0, 0.05);
        }
        
        .menu-trigger:focus {
            outline: 2px solid #3b82f6;
            outline-offset: 2px;
        }
        
        .menu-trigger:disabled {
            opacity: 0.5;
            cursor: not-allowed;
        }
        
        .menu-popup {
            position: absolute;
            z-index: 50;
            min-width: 200px;
            background: white;
            border: 1px solid #e5e7eb;
            border-radius: 6px;
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
            overflow: hidden;
        }
        
        .menu-popup[data-state="closed"] {
            opacity: 0;
            transform: scale(0.95);
            pointer-events: none;
        }
        
        .menu-popup[data-state="open"] {
            opacity: 1;
            transform: scale(1);
        }
        
        .menu-item {
            display: flex;
            align-items: center;
            padding: 8px 12px;
            min-height: 36px;
            cursor: default;
            transition: background-color 0.2s;
        }
        
        .menu-item[data-highlighted="true"] {
            background-color: #f3f4f6;
            color: #111827;
        }
        
        .menu-item[data-disabled="true"] {
            opacity: 0.5;
            cursor: not-allowed;
        }
        
        .menu-item[aria-selected="true"] {
            font-weight: 500;
        }
        
        .menu-separator {
            height: 1px;
            background-color: #e5e7eb;
            margin: 4px 8px;
        }
        
        .menu-group {
            margin: 4px 0;
        }
        
        .menu-group-heading {
            padding: 8px 12px;
            font-weight: 500;
            font-size: 12px;
            text-transform: uppercase;
            color: #6b7280;
        }
        
        .menu-checkbox {
            width: 16px;
            height: 16px;
            border: 1px solid #d1d5db;
            border-radius: 2px;
            margin-right: 8px;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        
        .menu-checkbox[checked="true"] {
            background-color: #3b82f6;
            border-color: #3b82f6;
            color: white;
        }
        
        .menu-radio {
            width: 16px;
            height: 16px;
            border: 1px solid #d1d5db;
            border-radius: 50%;
            margin-right: 8px;
            position: relative;
        }
        
        .menu-radio[checked="true"]::after {
            content: '';
            width: 8px;
            height: 8px;
            background-color: #3b82f6;
            border-radius: 50%;
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
        }
        
        .menu-backdrop {
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background-color: rgba(0, 0, 0, 0.5);
            z-index: 40;
        }
    "#;
    
    /// Menu event key codes
    pub const MENU_KEY_CODES: &[(&str, &str)] = &[
        ("Enter", "select"),
        (" ", "select"),
        ("ArrowDown", "next"),
        ("ArrowUp", "previous"),
        ("Home", "first"),
        ("End", "last"),
        ("Escape", "close"),
        ("Tab", "close"),
    ];
    
    /// Menu ARIA roles
    pub const MENU_ARIA_ROLES: [&str; 5] = [
        "menu",
        "menuitem", 
        "menuitemcheckbox",
        "menuitemradio",
        "separator",
    ];
    
    /// Menu positioning defaults
    pub const MENU_POSITIONING: &str = "bottom";
    pub const MENU_ALIGNMENT: &str = "left";
}