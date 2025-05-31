// shim.js

class MyFormData {

}

globalThis.FormData = MyFormData;


class CSSStyleDeclaration {
    getPropertyValue() {
        return 0; // IDK, figure this out later
    }
}

class Element {
    constructor() {
        this.style = new CSS2Properties();
    }
}

class HTMLElement extends Element { }

class CSS2Properties { }

// I'm not really sure what this thing is or does
class HTMLHtmlElement extends HTMLElement { }

class HTMLDocument {
    constructor() {
        this.documentElement = new HTMLHtmlElement();
    }

    createElement(element, options=null) {
        return new Element(); // Todo, maybe more specificity?
    }
}

class DOMParser { }
class PIXIShader { }
class PIXIContainer { }
class PIXIColor { }
class PIXIPoint { }
class PIXIMesh { }
class PIXIPolygon { }

globalThis.document = new HTMLDocument();
globalThis.getComputedStyle = (...args) => new CSSStyleDeclaration();
globalThis.DOMParser = DOMParser;
globalThis.HTMLElement = HTMLElement;
globalThis.HTMLHtmlElement = HTMLHtmlElement;
globalThis.PIXI = {
    Shader: PIXIShader,
    Container: PIXIContainer,
    Color: PIXIColor,
    Point: PIXIPoint,
    Mesh: PIXIMesh,
    Polygon: PIXIPolygon,
    settings: {
        PRECISION_VERTEX: null,
    }
}