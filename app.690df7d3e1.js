let wasm_bindgen = (function(exports) {
    let script_src;
    if (typeof document !== 'undefined' && document.currentScript !== null) {
        script_src = new URL(document.currentScript.src, location.href).toString();
    }

    /**
     * @param {string} key
     * @param {number} t
     * @returns {boolean}
     */
    function a11y_activate(key, t) {
        const val2 = +t;
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.a11y_activate(ptr0, len0, val2);
        return ret !== 0;
    }
    exports.a11y_activate = a11y_activate;

    /**
     * @param {string} key
     * @param {number} t
     */
    function a11y_focus(key, t) {
        const val2 = +t;
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.a11y_focus(ptr0, len0, val2);
    }
    exports.a11y_focus = a11y_focus;

    /**
     * @returns {string}
     */
    function a11y_update() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.a11y_update();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.a11y_update = a11y_update;

    /**
     * @param {string} name
     * @param {Uint8Array | number[]} bytes
     * @param {number} t
     */
    function add_asset(name, bytes, t) {
        const val3 = +t;
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.add_asset(ptr0, len0, ptr1, len1, val3);
    }
    exports.add_asset = add_asset;

    /**
     * @param {string} name
     * @param {Uint8Array | number[]} bytes
     * @param {number} t
     * @returns {number}
     */
    function add_face(name, bytes, t) {
        const val3 = +t;
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        const ret = wasm.add_face(ptr0, len0, ptr1, len1, val3);
        return ret;
    }
    exports.add_face = add_face;

    /**
     * The resources asked for that have not come, as a JSON list of names.
     * @returns {string}
     */
    function assets_wanted() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.assets_wanted();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.assets_wanted = assets_wanted;

    /**
     * The graphics backend in use: `webgpu` or `webgl2`.
     * @returns {string}
     */
    function backend() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.backend();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.backend = backend;

    /**
     * @returns {string | undefined}
     */
    function copy() {
        const ret = wasm.copy();
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]);
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    exports.copy = copy;

    /**
     * The CSS cursor for where the pointer is.
     * @returns {string}
     */
    function cursor() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.cursor();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.cursor = cursor;

    /**
     * @param {number} t
     * @returns {string | undefined}
     */
    function cut(t) {
        const ret = wasm.cut(t);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]);
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    exports.cut = cut;

    /**
     * @param {number} t
     */
    function drop_done(t) {
        wasm.drop_done(t);
    }
    exports.drop_done = drop_done;

    /**
     * @param {string} name
     * @param {Uint8Array | number[]} bytes
     */
    function drop_file(name, bytes) {
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.drop_file(ptr0, len0, ptr1, len1);
    }
    exports.drop_file = drop_file;

    /**
     * The faces the page uses that have not come, as a JSON list of names: fetch each, then `add_face`.
     * @returns {string}
     */
    function faces_wanted() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.faces_wanted();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.faces_wanted = faces_wanted;

    /**
     * @param {boolean} on
     */
    function files_hovering(on) {
        wasm.files_hovering(on);
    }
    exports.files_hovering = files_hovering;

    /**
     * @returns {boolean}
     */
    function flinging() {
        const ret = wasm.flinging();
        return ret !== 0;
    }
    exports.flinging = flinging;

    /**
     * @param {number} t
     * @returns {number}
     */
    function frame(t) {
        const ret = wasm.frame(t);
        return ret;
    }
    exports.frame = frame;

    /**
     * @returns {Float32Array}
     */
    function frame_costs() {
        const ret = wasm.frame_costs();
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    exports.frame_costs = frame_costs;

    /**
     * How the last frame was drawn (`perf::FramePath::line`): whether it moved the last frame and, if
     * not, why; whether the material pass was the specialised one; the share shaded; the scales.
     * @returns {string}
     */
    function frame_path() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.frame_path();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.frame_path = frame_path;

    /**
     * @returns {Uint32Array}
     */
    function frames() {
        const ret = wasm.frames();
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    exports.frames = frames;

    /**
     * How many glyphs the page draws (on home, the body is made of).
     * @returns {number}
     */
    function glyphs() {
        const ret = wasm.glyphs();
        return ret >>> 0;
    }
    exports.glyphs = glyphs;

    /**
     * @returns {string}
     */
    function hash() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hash();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.hash = hash;

    /**
     * The home page's state for the address (`source`, `body/cloud/08`, empty for the defaults).
     * @returns {string}
     */
    function hero() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.hero();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.hero = hero;

    /**
     * The body's circle on the canvas (centre x, y and radius, CSS px), empty while it is the listing:
     * a touch that starts inside it turns the body instead of scrolling the page.
     * @returns {Float32Array}
     */
    function hero_circle() {
        const ret = wasm.hero_circle();
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    exports.hero_circle = hero_circle;

    /**
     * The rail row: the four states, `play`, `skip`, `seed+`, `seed-`; and `hidden` / `shown` when the
     * tab or the stage goes out of view and comes back (nothing turns by itself out of view).
     * @param {string} what
     * @param {number} t
     */
    function hero_press(what, t) {
        const val2 = +t;
        const ptr0 = passStringToWasm0(what, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.hero_press(ptr0, len0, val2);
    }
    exports.hero_press = hero_press;

    /**
     * The home page's seed, for the rail row on a phone.
     * @returns {number}
     */
    function hero_seed() {
        const ret = wasm.hero_seed();
        return ret >>> 0;
    }
    exports.hero_seed = hero_seed;

    /**
     * The walk (0 none, 1 playing, 2 stopped by a person), whether it has been seen through, and the
     * state on the stage (1 to 4).
     * @returns {Uint32Array}
     */
    function hero_walk() {
        const ret = wasm.hero_walk();
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    exports.hero_walk = hero_walk;

    /**
     * @param {string} text
     * @param {number} t
     * @returns {boolean}
     */
    function ime_commit(text, t) {
        const val2 = +t;
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ime_commit(ptr0, len0, val2);
        return ret !== 0;
    }
    exports.ime_commit = ime_commit;

    /**
     * @param {string} text
     * @param {number} t
     * @returns {boolean}
     */
    function ime_preedit(text, t) {
        const val2 = +t;
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ime_preedit(ptr0, len0, val2);
        return ret !== 0;
    }
    exports.ime_preedit = ime_preedit;

    /**
     * @returns {Float32Array}
     */
    function ime_rect() {
        const ret = wasm.ime_rect();
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    exports.ime_rect = ime_rect;

    /**
     * @param {string} name
     * @param {boolean} shift
     * @param {number} t
     * @returns {boolean}
     */
    function key(name, shift, t) {
        const val2 = +shift;
        const val3 = +t;
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.key(ptr0, len0, val2, val3);
        return ret !== 0;
    }
    exports.key = key;

    /**
     * @param {string} name
     * @param {boolean} shift
     * @param {boolean} cmd
     * @param {boolean} alt
     * @param {number} t
     * @returns {boolean}
     */
    function key_mods(name, shift, cmd, alt, t) {
        const val2 = +shift;
        const val3 = +cmd;
        const val4 = +alt;
        const val5 = +t;
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.key_mods(ptr0, len0, val2, val3, val4, val5);
        return ret !== 0;
    }
    exports.key_mods = key_mods;

    /**
     * @param {string} name
     * @param {number} t
     */
    function key_up(name, t) {
        const val2 = +t;
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.key_up(ptr0, len0, val2);
    }
    exports.key_up = key_up;

    /**
     * @param {string} key
     * @returns {Float32Array}
     */
    function node_rect(key) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.node_rect(ptr0, len0);
        var v2 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v2;
    }
    exports.node_rect = node_rect;

    /**
     * Pace what moves by itself on the current page (`Ui::ambient_fps`): 30 frames a second on a
     * page of Works (M.7), no pacing elsewhere. The page calls it when its path changes.
     */
    function pace() {
        wasm.pace();
    }
    exports.pace = pace;

    /**
     * @param {string} text
     * @param {number} t
     * @returns {boolean}
     */
    function paste(text, t) {
        const val2 = +t;
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.paste(ptr0, len0, val2);
        return ret !== 0;
    }
    exports.paste = paste;

    /**
     * @returns {string}
     */
    function path() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.path();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.path = path;

    /**
     * Make the pictures the page's scene draws that the renderer does not have yet, each once. The
     * page calls it after a route and after every frame; when it made one it returns true and the
     * page draws that frame again at once, each new picture where it is, so no picture is ever seen
     * blank. Nothing new: false, and nothing is drawn again.
     * @returns {boolean}
     */
    function pictures() {
        const ret = wasm.pictures();
        return ret !== 0;
    }
    exports.pictures = pictures;

    /**
     * @param {number} id
     * @param {number} t
     */
    function pointer_cancel(id, t) {
        wasm.pointer_cancel(id, t);
    }
    exports.pointer_cancel = pointer_cancel;

    /**
     * @param {number} id
     * @param {number} k
     * @param {number} x
     * @param {number} y
     * @param {boolean} shift
     * @param {number} t
     * @param {number} te
     */
    function pointer_down(id, k, x, y, shift, t, te) {
        wasm.pointer_down(id, k, x, y, shift, t, te);
    }
    exports.pointer_down = pointer_down;

    /**
     * @param {number} t
     */
    function pointer_leave(t) {
        wasm.pointer_leave(t);
    }
    exports.pointer_leave = pointer_leave;

    /**
     * @param {number} id
     * @param {number} k
     * @param {number} x
     * @param {number} y
     * @param {number} t
     * @param {number} te
     */
    function pointer_move(id, k, x, y, t, te) {
        wasm.pointer_move(id, k, x, y, t, te);
    }
    exports.pointer_move = pointer_move;

    /**
     * @param {number} id
     * @param {number} k
     * @param {number} x
     * @param {number} y
     * @param {number} t
     */
    function pointer_up(id, k, x, y, t) {
        wasm.pointer_up(id, k, x, y, t);
    }
    exports.pointer_up = pointer_up;

    /**
     * Where the next in-app navigation was pressed (CSS px on the canvas; a link's centre for a
     * keyboard activation): its front starts there.
     * @param {number} x
     * @param {number} y
     */
    function pressed_at(x, y) {
        wasm.pressed_at(x, y);
    }
    exports.pressed_at = pressed_at;

    /**
     * A text the module does not carry (`texts`), given by the page: the view is laid out again.
     * @param {string} name
     * @param {string} body
     */
    function put_text(name, body) {
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(body, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.put_text(ptr0, len0, ptr1, len1);
    }
    exports.put_text = put_text;

    /**
     * The material resolution the readout's quality choice sets: 0 automatic, else a level of `perf::LEVELS`.
     * @returns {number}
     */
    function quality() {
        const ret = wasm.quality();
        return ret;
    }
    exports.quality = quality;

    /**
     * The readout's choice: 0 hidden, 1 shown, 2 with more (the page keeps it in localStorage).
     * @returns {number}
     */
    function readout() {
        const ret = wasm.readout();
        return ret;
    }
    exports.readout = readout;

    /**
     * A new route: the stages' scale steps from the device's own again.
     */
    function reset_stage() {
        wasm.reset_stage();
    }
    exports.reset_stage = reset_stage;

    /**
     * @param {number} w
     * @param {number} h
     * @param {number} scale
     */
    function resize(w, h, scale) {
        wasm.resize(w, h, scale);
    }
    exports.resize = resize;

    /**
     * @param {string} hash
     * @returns {boolean}
     */
    function route(hash) {
        const ptr0 = passStringToWasm0(hash, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.route(ptr0, len0);
        return ret !== 0;
    }
    exports.route = route;

    /**
     * @param {string} key
     * @returns {number}
     */
    function scroll_offset(key) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.scroll_offset(ptr0, len0);
        return ret;
    }
    exports.scroll_offset = scroll_offset;

    /**
     * @param {boolean} dark
     * @param {boolean} high_contrast
     */
    function set_appearance(dark, high_contrast) {
        wasm.set_appearance(dark, high_contrast);
    }
    exports.set_appearance = set_appearance;

    /**
     * Open a state from the address at rest; an empty one plays the short path when `intro` is set.
     * @param {string} state
     * @param {number} walk
     */
    function set_hero(state, walk) {
        const val2 = +walk;
        const ptr0 = passStringToWasm0(state, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.set_hero(ptr0, len0, val2);
    }
    exports.set_hero = set_hero;

    /**
     * Whether the pointer can hover: without (touch), the Lab index runs its centred cell.
     * @param {boolean} on
     */
    function set_hover(on) {
        wasm.set_hover(on);
    }
    exports.set_hover = set_hover;

    /**
     * @param {string} tag
     */
    function set_locale(tag) {
        const ptr0 = passStringToWasm0(tag, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.set_locale(ptr0, len0);
    }
    exports.set_locale = set_locale;

    /**
     * @param {string} theme
     * @param {string} material
     */
    function set_look(theme, material) {
        let ptr0, len0, ptr1, len1;
        try {
            ptr0 = passStringToWasm0(theme, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passStringToWasm0(material, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len1 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.set_look(ptr0, len0, ptr1, len1);
    }
    exports.set_look = set_look;

    /**
     * @param {number} q
     */
    function set_quality(q) {
        wasm.set_quality(q);
    }
    exports.set_quality = set_quality;

    /**
     * @param {number} v
     */
    function set_readout(v) {
        wasm.set_readout(v);
    }
    exports.set_readout = set_readout;

    /**
     * @param {boolean} on
     */
    function set_reduced_motion(on) {
        wasm.set_reduced_motion(on);
    }
    exports.set_reduced_motion = set_reduced_motion;

    /**
     * The scheme the page is in (AA.10): paper (light) or graphite (dark). A change made with the bar's
     * switch comes with where it was pressed (CSS px on the canvas) and is a commit from there; one
     * made by the system (NaN) is a cut. Either way only the look's colours change.
     * @param {boolean} paper
     * @param {number} x
     * @param {number} y
     */
    function set_scheme(paper, x, y) {
        wasm.set_scheme(paper, x, y);
    }
    exports.set_scheme = set_scheme;

    /**
     * The home page's stage (CSS px from the canvas's top left), laid out by the page, and the optical
     * centre of its text block (vertical, same space; negative for none); a zero-sized stage gives the
     * scene's own.
     * @param {number} x0
     * @param {number} y0
     * @param {number} x1
     * @param {number} y1
     * @param {number} centre
     */
    function set_stage(x0, y0, x1, y1, centre) {
        wasm.set_stage(x0, y0, x1, y1, centre);
    }
    exports.set_stage = set_stage;

    /**
     * The page has the HTML status line: it carries the one-line readout's numbers (AA.6).
     * @param {boolean} on
     */
    function set_status_line(on) {
        wasm.set_status_line(on);
    }
    exports.set_status_line = set_status_line;

    /**
     * Force the material pass made for the page's material on (true) or off (false), or leave it to
     * the renderer (null), from the next frame (`Renderer::spec_force`), for an A/B on one page.
     * @param {boolean | null} [on]
     */
    function spec_force(on) {
        wasm.spec_force(isLikeNone(on) ? 0xFFFFFF : on ? 1 : 0);
    }
    exports.spec_force = spec_force;

    /**
     * The stages' scale while it is below the device's (`stage 1.5x`), empty at the device's own.
     * @returns {string}
     */
    function stage() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.stage();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.stage = stage;

    /**
     * @param {HTMLCanvasElement} canvas
     * @param {Uint8Array | number[]} sans
     * @param {Uint8Array | number[]} pixel
     * @param {Uint8Array | number[]} mono
     * @param {number} scale
     * @param {string} hash
     * @returns {Promise<void>}
     */
    function start(canvas, sans, pixel, mono, scale, hash) {
        const val5 = +scale;
        let ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3;
        try {
            ptr0 = passArray8ToWasm0(sans, wasm.__wbindgen_malloc);
            len0 = WASM_VECTOR_LEN;
            ptr1 = passArray8ToWasm0(pixel, wasm.__wbindgen_malloc);
            len1 = WASM_VECTOR_LEN;
            ptr2 = passArray8ToWasm0(mono, wasm.__wbindgen_malloc);
            len2 = WASM_VECTOR_LEN;
            ptr3 = passStringToWasm0(hash, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            len3 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr3) { wasm.__wbindgen_free(ptr3, len3 * 1, 1); }
            if (ptr2) { wasm.__wbindgen_free(ptr2, len2 * 1, 1); }
            if (ptr1) { wasm.__wbindgen_free(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_free(ptr0, len0 * 1, 1); }
            throw e;
        }
        const ret = wasm.start(canvas, ptr0, len0, ptr1, len1, ptr2, len2, val5, ptr3, len3);
        return ret;
    }
    exports.start = start;

    /**
     * Why `start` failed (`StartError::kind`: `adapter`, `device`), or empty.
     * @returns {string}
     */
    function start_failure() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.start_failure();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.start_failure = start_failure;

    /**
     * Text a copy button asked to put on the clipboard (once).
     * @returns {string | undefined}
     */
    function take_clipboard() {
        const ret = wasm.take_clipboard();
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]);
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    exports.take_clipboard = take_clipboard;

    /**
     * An HTML page a press asked for (a piece's recipe), once.
     * @returns {string}
     */
    function take_navigate() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.take_navigate();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.take_navigate = take_navigate;

    /**
     * @returns {boolean}
     */
    function text_focused() {
        const ret = wasm.text_focused();
        return ret !== 0;
    }
    exports.text_focused = text_focused;

    /**
     * @param {string} text
     * @param {number} t
     * @returns {boolean}
     */
    function text_input(text, t) {
        const val2 = +t;
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.text_input(ptr0, len0, val2);
        return ret !== 0;
    }
    exports.text_input = text_input;

    /**
     * The current page's name, for the document's title.
     * @returns {string}
     */
    function title() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.title();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.title = title;

    /**
     * The texts the module has asked for since the last call, one a line, for the page to fetch.
     * @returns {string}
     */
    function wanted_texts() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wanted_texts();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    exports.wanted_texts = wanted_texts;

    /**
     * How far the warm-up is, `[done, total]`; empty once it is done.
     * @returns {Uint32Array}
     */
    function warm() {
        const ret = wasm.warm();
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    exports.warm = warm;

    /**
     * The warm-up in idle time: 1 while pieces are left, plus 2 when a frame is wanted now. A call
     * that finds the module busy says 1, so the page asks again.
     * @param {number} budget_ms
     * @param {number} t
     * @returns {number}
     */
    function warm_idle(budget_ms, t) {
        const ret = wasm.warm_idle(budget_ms, t);
        return ret >>> 0;
    }
    exports.warm_idle = warm_idle;

    /**
     * @param {number} x
     * @param {number} y
     * @param {number} dx
     * @param {number} dy
     * @param {number} t
     */
    function wheel(x, y, dx, dy, t) {
        wasm.wheel(x, y, dx, dy, t);
    }
    exports.wheel = wheel;

    /**
     * The current page's path under the site's base (`components/button/`; empty for home).
     * A piece of Works, for the page around the canvas: [the bar's tone (0 none, 1 light, 2 dark),
     * the chrome faded (1), the piece resting (1), the status line's tone].
     * @returns {Uint8Array}
     */
    function works_state() {
        const ret = wasm.works_state();
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    exports.works_state = works_state;
    function __wbg_get_imports() {
        const import0 = {
            __proto__: null,
            __wbg_Error_208446807d0d1e30: function(arg0, arg1) {
                const ret = Error(getStringFromWasm0(arg0, arg1));
                return ret;
            },
            __wbg_Window_41838cbcc08822b5: function(arg0) {
                const ret = arg0.Window;
                return ret;
            },
            __wbg_WorkerGlobalScope_448e3830f27a30fd: function(arg0) {
                const ret = arg0.WorkerGlobalScope;
                return ret;
            },
            __wbg___wbindgen_boolean_get_4f1baf9a00be3c74: function(arg0) {
                const v = arg0;
                const ret = typeof(v) === 'boolean' ? v : undefined;
                return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
            },
            __wbg___wbindgen_debug_string_f249a2a76caa9a55: function(arg0, arg1) {
                const ret = debugString(arg1);
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_is_function_741ff2bb612836f5: function(arg0) {
                const ret = typeof(arg0) === 'function';
                return ret;
            },
            __wbg___wbindgen_is_null_c22311b1cc228e3a: function(arg0) {
                const ret = arg0 === null;
                return ret;
            },
            __wbg___wbindgen_is_string_fc456fee5d3317de: function(arg0) {
                const ret = typeof(arg0) === 'string';
                return ret;
            },
            __wbg___wbindgen_is_undefined_be5b34249ff1fc15: function(arg0) {
                const ret = arg0 === undefined;
                return ret;
            },
            __wbg___wbindgen_number_get_934b640d0889af78: function(arg0, arg1) {
                const obj = arg1;
                const ret = typeof(obj) === 'number' ? obj : undefined;
                getDataViewMemory0().setFloat64((arg0 >>> 0) + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, !isLikeNone(ret), true);
            },
            __wbg___wbindgen_string_get_7ce8eee7fd327545: function(arg0, arg1) {
                const obj = arg1;
                const ret = typeof(obj) === 'string' ? obj : undefined;
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
            },
            __wbg___wbindgen_throw_4c88d8bf264fd984: function(arg0, arg1) {
                throw new Error(getStringFromWasm0(arg0, arg1));
            },
            __wbg__wbg_cb_unref_09736879a3f67559: function(arg0) {
                arg0._wbg_cb_unref();
            },
            __wbg_activeTexture_45b425a49df733ce: function(arg0, arg1) {
                arg0.activeTexture(arg1 >>> 0);
            },
            __wbg_activeTexture_89cef8c297ad0698: function(arg0, arg1) {
                arg0.activeTexture(arg1 >>> 0);
            },
            __wbg_attachShader_3ed301cc92db59f6: function(arg0, arg1, arg2) {
                arg0.attachShader(arg1, arg2);
            },
            __wbg_attachShader_817dcec9e79fd18f: function(arg0, arg1, arg2) {
                arg0.attachShader(arg1, arg2);
            },
            __wbg_beginComputePass_b3bb1a69e6a70bde: function(arg0, arg1) {
                const ret = arg0.beginComputePass(arg1);
                return ret;
            },
            __wbg_beginQuery_f800947e639433e8: function(arg0, arg1, arg2) {
                arg0.beginQuery(arg1 >>> 0, arg2);
            },
            __wbg_beginRenderPass_ac30593a03c9a74e: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.beginRenderPass(arg1);
                return ret;
            }, arguments); },
            __wbg_bindAttribLocation_40c41e6185822b52: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.bindAttribLocation(arg1, arg2 >>> 0, getStringFromWasm0(arg3, arg4));
            },
            __wbg_bindAttribLocation_b51bf2d65ded8f70: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.bindAttribLocation(arg1, arg2 >>> 0, getStringFromWasm0(arg3, arg4));
            },
            __wbg_bindBufferRange_da5accbcd5d223ad: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                arg0.bindBufferRange(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
            },
            __wbg_bindBuffer_b3bb001fdbee089a: function(arg0, arg1, arg2) {
                arg0.bindBuffer(arg1 >>> 0, arg2);
            },
            __wbg_bindBuffer_cbc9075f76fffa51: function(arg0, arg1, arg2) {
                arg0.bindBuffer(arg1 >>> 0, arg2);
            },
            __wbg_bindFramebuffer_79534f24b57b7b95: function(arg0, arg1, arg2) {
                arg0.bindFramebuffer(arg1 >>> 0, arg2);
            },
            __wbg_bindFramebuffer_b245cd93a150dd96: function(arg0, arg1, arg2) {
                arg0.bindFramebuffer(arg1 >>> 0, arg2);
            },
            __wbg_bindRenderbuffer_3458a69a6dc55057: function(arg0, arg1, arg2) {
                arg0.bindRenderbuffer(arg1 >>> 0, arg2);
            },
            __wbg_bindRenderbuffer_612f27d9b3b97b9f: function(arg0, arg1, arg2) {
                arg0.bindRenderbuffer(arg1 >>> 0, arg2);
            },
            __wbg_bindSampler_0fb033a6ffd5d7b3: function(arg0, arg1, arg2) {
                arg0.bindSampler(arg1 >>> 0, arg2);
            },
            __wbg_bindTexture_6f822943b2790466: function(arg0, arg1, arg2) {
                arg0.bindTexture(arg1 >>> 0, arg2);
            },
            __wbg_bindTexture_a23350e1b573ddfd: function(arg0, arg1, arg2) {
                arg0.bindTexture(arg1 >>> 0, arg2);
            },
            __wbg_bindVertexArrayOES_5161c1f5b1fea3f9: function(arg0, arg1) {
                arg0.bindVertexArrayOES(arg1);
            },
            __wbg_bindVertexArray_cc64c48bf67ab8d4: function(arg0, arg1) {
                arg0.bindVertexArray(arg1);
            },
            __wbg_blendColor_8631cab46394c93b: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.blendColor(arg1, arg2, arg3, arg4);
            },
            __wbg_blendColor_957c8ddad53063a2: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.blendColor(arg1, arg2, arg3, arg4);
            },
            __wbg_blendEquationSeparate_0dceb23863585a87: function(arg0, arg1, arg2) {
                arg0.blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
            },
            __wbg_blendEquationSeparate_156fe040a9aebfd6: function(arg0, arg1, arg2) {
                arg0.blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
            },
            __wbg_blendEquation_847eb360af3b2786: function(arg0, arg1) {
                arg0.blendEquation(arg1 >>> 0);
            },
            __wbg_blendEquation_cd72c0ac181cd88d: function(arg0, arg1) {
                arg0.blendEquation(arg1 >>> 0);
            },
            __wbg_blendFuncSeparate_6db2212299fd7f1e: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
            },
            __wbg_blendFuncSeparate_eb9098d760f4a25c: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
            },
            __wbg_blendFunc_378a3be8ab6cb7d0: function(arg0, arg1, arg2) {
                arg0.blendFunc(arg1 >>> 0, arg2 >>> 0);
            },
            __wbg_blendFunc_68b4cfb9bd98a13f: function(arg0, arg1, arg2) {
                arg0.blendFunc(arg1 >>> 0, arg2 >>> 0);
            },
            __wbg_blitFramebuffer_d17b7bddf5e86dbc: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
                arg0.blitFramebuffer(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0);
            },
            __wbg_bufferData_2fb8740d60478bf0: function(arg0, arg1, arg2, arg3) {
                arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
            },
            __wbg_bufferData_41b591a9f2d502ad: function(arg0, arg1, arg2, arg3) {
                arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
            },
            __wbg_bufferData_9305dba3ba9cba4e: function(arg0, arg1, arg2, arg3) {
                arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
            },
            __wbg_bufferData_9eb9c4c5053ac480: function(arg0, arg1, arg2, arg3) {
                arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
            },
            __wbg_bufferSubData_8acfdcd4b9f7edeb: function(arg0, arg1, arg2, arg3) {
                arg0.bufferSubData(arg1 >>> 0, arg2, arg3);
            },
            __wbg_bufferSubData_9bec93d67e98757e: function(arg0, arg1, arg2, arg3) {
                arg0.bufferSubData(arg1 >>> 0, arg2, arg3);
            },
            __wbg_call_a410eb4b59ed5965: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.call(arg1, arg2);
                return ret;
            }, arguments); },
            __wbg_clearBufferfv_474b28c18db287a8: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.clearBufferfv(arg1 >>> 0, arg2, getArrayF32FromWasm0(arg3, arg4));
            },
            __wbg_clearBufferiv_a89cfdf7da6f2819: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.clearBufferiv(arg1 >>> 0, arg2, getArrayI32FromWasm0(arg3, arg4));
            },
            __wbg_clearBufferuiv_e723936f302725de: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.clearBufferuiv(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4));
            },
            __wbg_clearDepth_3417be30229e823a: function(arg0, arg1) {
                arg0.clearDepth(arg1);
            },
            __wbg_clearDepth_3dc1f52f96f29526: function(arg0, arg1) {
                arg0.clearDepth(arg1);
            },
            __wbg_clearStencil_29f687031586f5e6: function(arg0, arg1) {
                arg0.clearStencil(arg1);
            },
            __wbg_clearStencil_59a8a925116b24ec: function(arg0, arg1) {
                arg0.clearStencil(arg1);
            },
            __wbg_clear_3ffa1ed794f92a04: function(arg0, arg1) {
                arg0.clear(arg1 >>> 0);
            },
            __wbg_clear_ccb3d618da90fea3: function(arg0, arg1) {
                arg0.clear(arg1 >>> 0);
            },
            __wbg_clientWaitSync_fee803cb72073198: function(arg0, arg1, arg2, arg3) {
                const ret = arg0.clientWaitSync(arg1, arg2 >>> 0, arg3 >>> 0);
                return ret;
            },
            __wbg_colorMask_24c1b6c3ec53f8c8: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
            },
            __wbg_colorMask_cfc2d619716d01ed: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
            },
            __wbg_compileShader_11eb6754e9b15970: function(arg0, arg1) {
                arg0.compileShader(arg1);
            },
            __wbg_compileShader_54c3771819e6caba: function(arg0, arg1) {
                arg0.compileShader(arg1);
            },
            __wbg_compressedTexSubImage2D_8706ea3fb9e0180c: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
                arg0.compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8);
            },
            __wbg_compressedTexSubImage2D_b6a8807f715c2782: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
                arg0.compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8, arg9);
            },
            __wbg_compressedTexSubImage2D_d66b16b23793038b: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
                arg0.compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8);
            },
            __wbg_compressedTexSubImage3D_74b7bfbc9a7fc90f: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
                arg0.compressedTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10, arg11);
            },
            __wbg_compressedTexSubImage3D_e1d361485803b713: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
                arg0.compressedTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10);
            },
            __wbg_configure_2f4cbe7746e5aa81: function() { return handleError(function (arg0, arg1) {
                arg0.configure(arg1);
            }, arguments); },
            __wbg_copyBufferSubData_b10635982222cbc4: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                arg0.copyBufferSubData(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
            },
            __wbg_copyTexSubImage2D_0cc80bc35bca1d8e: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
                arg0.copyTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
            },
            __wbg_copyTexSubImage2D_be50078e0f0575f1: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
                arg0.copyTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
            },
            __wbg_copyTexSubImage3D_e35f615c7c8244ba: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
                arg0.copyTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);
            },
            __wbg_copyTextureToTexture_aa278b6e539310a3: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                arg0.copyTextureToTexture(arg1, arg2, arg3);
            }, arguments); },
            __wbg_createBindGroupLayout_03a20e72b41b406e: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.createBindGroupLayout(arg1);
                return ret;
            }, arguments); },
            __wbg_createBindGroup_ec4dc4d9e46e1104: function(arg0, arg1) {
                const ret = arg0.createBindGroup(arg1);
                return ret;
            },
            __wbg_createBuffer_2c27ad5f899d418d: function(arg0) {
                const ret = arg0.createBuffer();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createBuffer_baa81dab839b6c5c: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.createBuffer(arg1);
                return ret;
            }, arguments); },
            __wbg_createBuffer_ca6ca40e9fb13678: function(arg0) {
                const ret = arg0.createBuffer();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createCommandEncoder_a5136f45baaba888: function(arg0, arg1) {
                const ret = arg0.createCommandEncoder(arg1);
                return ret;
            },
            __wbg_createComputePipeline_6a05383609fddac6: function(arg0, arg1) {
                const ret = arg0.createComputePipeline(arg1);
                return ret;
            },
            __wbg_createFramebuffer_cdf686778c61014d: function(arg0) {
                const ret = arg0.createFramebuffer();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createFramebuffer_f52399034cb1a200: function(arg0) {
                const ret = arg0.createFramebuffer();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createPipelineLayout_a955ff1e35a63634: function(arg0, arg1) {
                const ret = arg0.createPipelineLayout(arg1);
                return ret;
            },
            __wbg_createProgram_de0881c688a8e43b: function(arg0) {
                const ret = arg0.createProgram();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createProgram_ea784724f6f0074f: function(arg0) {
                const ret = arg0.createProgram();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createQuery_8f0481bdf42276b8: function(arg0) {
                const ret = arg0.createQuery();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createRenderPipeline_f31fc336274aaab2: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.createRenderPipeline(arg1);
                return ret;
            }, arguments); },
            __wbg_createRenderbuffer_19c2eec619a37a15: function(arg0) {
                const ret = arg0.createRenderbuffer();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createRenderbuffer_e45c14c918a72668: function(arg0) {
                const ret = arg0.createRenderbuffer();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createSampler_2fe8901be471a83a: function(arg0, arg1) {
                const ret = arg0.createSampler(arg1);
                return ret;
            },
            __wbg_createSampler_5bc030b64ed02c52: function(arg0) {
                const ret = arg0.createSampler();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createShaderModule_f540847678b68e08: function(arg0, arg1) {
                const ret = arg0.createShaderModule(arg1);
                return ret;
            },
            __wbg_createShader_9754a1e4ef374e84: function(arg0, arg1) {
                const ret = arg0.createShader(arg1 >>> 0);
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createShader_d2d90020273cef2b: function(arg0, arg1) {
                const ret = arg0.createShader(arg1 >>> 0);
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createTexture_3287f26407473071: function(arg0) {
                const ret = arg0.createTexture();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createTexture_5e2361f396945928: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.createTexture(arg1);
                return ret;
            }, arguments); },
            __wbg_createTexture_eec9a1994aabeca7: function(arg0) {
                const ret = arg0.createTexture();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createVertexArrayOES_3eaf087ef22da1cb: function(arg0) {
                const ret = arg0.createVertexArrayOES();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createVertexArray_1293d2ffa8346be2: function(arg0) {
                const ret = arg0.createVertexArray();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_createView_e495f88700d28104: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.createView(arg1);
                return ret;
            }, arguments); },
            __wbg_cullFace_208151a46885a1fa: function(arg0, arg1) {
                arg0.cullFace(arg1 >>> 0);
            },
            __wbg_cullFace_9eaca57652d7ed37: function(arg0, arg1) {
                arg0.cullFace(arg1 >>> 0);
            },
            __wbg_deleteBuffer_673584ba5a55669d: function(arg0, arg1) {
                arg0.deleteBuffer(arg1);
            },
            __wbg_deleteBuffer_ffa760fd3b0b5ef1: function(arg0, arg1) {
                arg0.deleteBuffer(arg1);
            },
            __wbg_deleteFramebuffer_0ec884f89db72a3e: function(arg0, arg1) {
                arg0.deleteFramebuffer(arg1);
            },
            __wbg_deleteFramebuffer_ffdb1bee4c92ae4c: function(arg0, arg1) {
                arg0.deleteFramebuffer(arg1);
            },
            __wbg_deleteProgram_8c38fff0f4a88dea: function(arg0, arg1) {
                arg0.deleteProgram(arg1);
            },
            __wbg_deleteProgram_c07d952a1b0c1cb6: function(arg0, arg1) {
                arg0.deleteProgram(arg1);
            },
            __wbg_deleteQuery_dfd665f6d8652e22: function(arg0, arg1) {
                arg0.deleteQuery(arg1);
            },
            __wbg_deleteRenderbuffer_3a6e1676ab926e0f: function(arg0, arg1) {
                arg0.deleteRenderbuffer(arg1);
            },
            __wbg_deleteRenderbuffer_895ff607251243a9: function(arg0, arg1) {
                arg0.deleteRenderbuffer(arg1);
            },
            __wbg_deleteSampler_2b73428131e403e1: function(arg0, arg1) {
                arg0.deleteSampler(arg1);
            },
            __wbg_deleteShader_320530dd636bc187: function(arg0, arg1) {
                arg0.deleteShader(arg1);
            },
            __wbg_deleteShader_ac74568b64a1c3c7: function(arg0, arg1) {
                arg0.deleteShader(arg1);
            },
            __wbg_deleteSync_eede535e5826f144: function(arg0, arg1) {
                arg0.deleteSync(arg1);
            },
            __wbg_deleteTexture_1653a9994a44215e: function(arg0, arg1) {
                arg0.deleteTexture(arg1);
            },
            __wbg_deleteTexture_89bd1d49ae4fb309: function(arg0, arg1) {
                arg0.deleteTexture(arg1);
            },
            __wbg_deleteVertexArrayOES_2214ecdf65be031d: function(arg0, arg1) {
                arg0.deleteVertexArrayOES(arg1);
            },
            __wbg_deleteVertexArray_2589b74b67ba20c8: function(arg0, arg1) {
                arg0.deleteVertexArray(arg1);
            },
            __wbg_depthFunc_04abd9f6ff980ba9: function(arg0, arg1) {
                arg0.depthFunc(arg1 >>> 0);
            },
            __wbg_depthFunc_584bfa0174e7628a: function(arg0, arg1) {
                arg0.depthFunc(arg1 >>> 0);
            },
            __wbg_depthMask_276856a569045897: function(arg0, arg1) {
                arg0.depthMask(arg1 !== 0);
            },
            __wbg_depthMask_2ba7e83178c3cec3: function(arg0, arg1) {
                arg0.depthMask(arg1 !== 0);
            },
            __wbg_depthRange_8b4b69a7c91ff92b: function(arg0, arg1, arg2) {
                arg0.depthRange(arg1, arg2);
            },
            __wbg_depthRange_942825e3232596de: function(arg0, arg1, arg2) {
                arg0.depthRange(arg1, arg2);
            },
            __wbg_description_359d32310adf317e: function(arg0, arg1) {
                const ret = arg1.description;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
            },
            __wbg_disableVertexAttribArray_011898581287ef1e: function(arg0, arg1) {
                arg0.disableVertexAttribArray(arg1 >>> 0);
            },
            __wbg_disableVertexAttribArray_5c66375eba552ee7: function(arg0, arg1) {
                arg0.disableVertexAttribArray(arg1 >>> 0);
            },
            __wbg_disable_2ead6c11cefd0322: function(arg0, arg1) {
                arg0.disable(arg1 >>> 0);
            },
            __wbg_disable_8a9e81e6d64d046b: function(arg0, arg1) {
                arg0.disable(arg1 >>> 0);
            },
            __wbg_dispatchWorkgroups_74a6ec8812ef9f3b: function(arg0, arg1, arg2, arg3) {
                arg0.dispatchWorkgroups(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
            },
            __wbg_document_3b5c3071b43e97ee: function(arg0) {
                const ret = arg0.document;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_drawArraysInstancedANGLE_635e10224985b842: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.drawArraysInstancedANGLE(arg1 >>> 0, arg2, arg3, arg4);
            },
            __wbg_drawArraysInstanced_762b36f2147c7a85: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.drawArraysInstanced(arg1 >>> 0, arg2, arg3, arg4);
            },
            __wbg_drawArrays_a9c18db2e94f4a82: function(arg0, arg1, arg2, arg3) {
                arg0.drawArrays(arg1 >>> 0, arg2, arg3);
            },
            __wbg_drawArrays_fe5a51410bfabb5b: function(arg0, arg1, arg2, arg3) {
                arg0.drawArrays(arg1 >>> 0, arg2, arg3);
            },
            __wbg_drawBuffersWEBGL_49c672d0ddcdfe71: function(arg0, arg1) {
                arg0.drawBuffersWEBGL(arg1);
            },
            __wbg_drawBuffers_e296659ce384e721: function(arg0, arg1) {
                arg0.drawBuffers(arg1);
            },
            __wbg_drawElementsInstancedANGLE_470ce8b489eeafa5: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                arg0.drawElementsInstancedANGLE(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
            },
            __wbg_drawElementsInstanced_3240c0c168d6af87: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                arg0.drawElementsInstanced(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
            },
            __wbg_draw_977efcdcd4b12838: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
            },
            __wbg_enableVertexAttribArray_d485706ade6bebf8: function(arg0, arg1) {
                arg0.enableVertexAttribArray(arg1 >>> 0);
            },
            __wbg_enableVertexAttribArray_eeecc6185af24227: function(arg0, arg1) {
                arg0.enableVertexAttribArray(arg1 >>> 0);
            },
            __wbg_enable_c3c00565c89d3175: function(arg0, arg1) {
                arg0.enable(arg1 >>> 0);
            },
            __wbg_enable_e955d456ef6b93f7: function(arg0, arg1) {
                arg0.enable(arg1 >>> 0);
            },
            __wbg_endQuery_f052a8f0f3589d3b: function(arg0, arg1) {
                arg0.endQuery(arg1 >>> 0);
            },
            __wbg_end_23ba48463344fbc0: function(arg0) {
                arg0.end();
            },
            __wbg_end_3bbedaa037606029: function(arg0) {
                arg0.end();
            },
            __wbg_error_2fa67a9496f77475: function(arg0) {
                const ret = arg0.error;
                return ret;
            },
            __wbg_error_757e9472f8410341: function(arg0, arg1) {
                let deferred0_0;
                let deferred0_1;
                try {
                    deferred0_0 = arg0;
                    deferred0_1 = arg1;
                    console.error(getStringFromWasm0(arg0, arg1));
                } finally {
                    wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
                }
            },
            __wbg_features_b650c8412d082ef1: function(arg0) {
                const ret = arg0.features;
                return ret;
            },
            __wbg_fenceSync_53f6f24f4bb43d24: function(arg0, arg1, arg2) {
                const ret = arg0.fenceSync(arg1 >>> 0, arg2 >>> 0);
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_finish_49529a046631ad9a: function(arg0) {
                const ret = arg0.finish();
                return ret;
            },
            __wbg_finish_678df663961386ff: function(arg0, arg1) {
                const ret = arg0.finish(arg1);
                return ret;
            },
            __wbg_finish_783de9bff3b56540: function(arg0) {
                arg0.finish();
            },
            __wbg_finish_abf9726b03095270: function(arg0) {
                arg0.finish();
            },
            __wbg_flush_4911c503a18d091d: function(arg0) {
                arg0.flush();
            },
            __wbg_flush_a9981e0bd1a44897: function(arg0) {
                arg0.flush();
            },
            __wbg_framebufferRenderbuffer_1c89563506a7d0ac: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4);
            },
            __wbg_framebufferRenderbuffer_ffaba93222aea77c: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4);
            },
            __wbg_framebufferTexture2D_2c69ac0ce67faf7d: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                arg0.framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5);
            },
            __wbg_framebufferTexture2D_b6250a0c37eb48c2: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                arg0.framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5);
            },
            __wbg_framebufferTextureLayer_2f6e24e2dbf9b214: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                arg0.framebufferTextureLayer(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
            },
            __wbg_framebufferTextureMultiviewOVR_63308e847d8b8f6c: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
                arg0.framebufferTextureMultiviewOVR(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5, arg6);
            },
            __wbg_frontFace_124bb11a73cc0dd1: function(arg0, arg1) {
                arg0.frontFace(arg1 >>> 0);
            },
            __wbg_frontFace_8bbd77ca3b8f90a6: function(arg0, arg1) {
                arg0.frontFace(arg1 >>> 0);
            },
            __wbg_getBufferSubData_4a9f8244f0e59e63: function(arg0, arg1, arg2, arg3) {
                arg0.getBufferSubData(arg1 >>> 0, arg2, arg3);
            },
            __wbg_getContext_1938edb9a972f63d: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            }, arguments); },
            __wbg_getContext_9d32151c28a36ed6: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = arg0.getContext(getStringFromWasm0(arg1, arg2), arg3);
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            }, arguments); },
            __wbg_getContext_d93f904b714852ae: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            }, arguments); },
            __wbg_getContext_f2ea37484e3f8ff4: function() { return handleError(function (arg0, arg1, arg2, arg3) {
                const ret = arg0.getContext(getStringFromWasm0(arg1, arg2), arg3);
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            }, arguments); },
            __wbg_getCurrentTexture_e9d52aabce56f3fb: function() { return handleError(function (arg0) {
                const ret = arg0.getCurrentTexture();
                return ret;
            }, arguments); },
            __wbg_getExtension_30c10f49f67eb05d: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.getExtension(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            }, arguments); },
            __wbg_getIndexedParameter_5d1c39ef8e171053: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.getIndexedParameter(arg1 >>> 0, arg2 >>> 0);
                return ret;
            }, arguments); },
            __wbg_getMappedRange_fd3092e276caa831: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.getMappedRange(arg1, arg2);
                return ret;
            }, arguments); },
            __wbg_getParameter_4c67158a3cef7a34: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.getParameter(arg1 >>> 0);
                return ret;
            }, arguments); },
            __wbg_getParameter_7fbe79a0114e119f: function() { return handleError(function (arg0, arg1) {
                const ret = arg0.getParameter(arg1 >>> 0);
                return ret;
            }, arguments); },
            __wbg_getPreferredCanvasFormat_f775f4ebe1768892: function(arg0) {
                const ret = arg0.getPreferredCanvasFormat();
                return (__wbindgen_enum_GpuTextureFormat.indexOf(ret) + 1 || 102) - 1;
            },
            __wbg_getProgramInfoLog_01799fd7b07b7f3c: function(arg0, arg1, arg2) {
                const ret = arg1.getProgramInfoLog(arg2);
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
            },
            __wbg_getProgramInfoLog_92d61551e1ab39ea: function(arg0, arg1, arg2) {
                const ret = arg1.getProgramInfoLog(arg2);
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
            },
            __wbg_getProgramParameter_1e7562e7d66170c3: function(arg0, arg1, arg2) {
                const ret = arg0.getProgramParameter(arg1, arg2 >>> 0);
                return ret;
            },
            __wbg_getProgramParameter_3edd526f9e5f6c10: function(arg0, arg1, arg2) {
                const ret = arg0.getProgramParameter(arg1, arg2 >>> 0);
                return ret;
            },
            __wbg_getQueryParameter_0beb5dc153d8ca4f: function(arg0, arg1, arg2) {
                const ret = arg0.getQueryParameter(arg1, arg2 >>> 0);
                return ret;
            },
            __wbg_getShaderInfoLog_3cfe5b3e097d0f2d: function(arg0, arg1, arg2) {
                const ret = arg1.getShaderInfoLog(arg2);
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
            },
            __wbg_getShaderInfoLog_8758e8aea7960121: function(arg0, arg1, arg2) {
                const ret = arg1.getShaderInfoLog(arg2);
                var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                var len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
            },
            __wbg_getShaderParameter_a346f74d5ba41c4b: function(arg0, arg1, arg2) {
                const ret = arg0.getShaderParameter(arg1, arg2 >>> 0);
                return ret;
            },
            __wbg_getShaderParameter_fc64069fbc010d19: function(arg0, arg1, arg2) {
                const ret = arg0.getShaderParameter(arg1, arg2 >>> 0);
                return ret;
            },
            __wbg_getSupportedExtensions_f48b337512e8f8ce: function(arg0) {
                const ret = arg0.getSupportedExtensions();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_getSupportedProfiles_b5cc35261935c0c9: function(arg0) {
                const ret = arg0.getSupportedProfiles();
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_getSyncParameter_8b7b36e9efd95806: function(arg0, arg1, arg2) {
                const ret = arg0.getSyncParameter(arg1, arg2 >>> 0);
                return ret;
            },
            __wbg_getUniformBlockIndex_0742ae35f926850a: function(arg0, arg1, arg2, arg3) {
                const ret = arg0.getUniformBlockIndex(arg1, getStringFromWasm0(arg2, arg3));
                return ret;
            },
            __wbg_getUniformLocation_6352e0669c8f5393: function(arg0, arg1, arg2, arg3) {
                const ret = arg0.getUniformLocation(arg1, getStringFromWasm0(arg2, arg3));
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_getUniformLocation_e0d11c86376cdd26: function(arg0, arg1, arg2, arg3) {
                const ret = arg0.getUniformLocation(arg1, getStringFromWasm0(arg2, arg3));
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_get_0c55869fe53c7d14: function(arg0, arg1) {
                const ret = arg0[arg1 >>> 0];
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_get_unchecked_e6325ad374eff861: function(arg0, arg1) {
                const ret = arg0[arg1 >>> 0];
                return ret;
            },
            __wbg_gpu_aa311a9494720027: function(arg0) {
                const ret = arg0.gpu;
                return ret;
            },
            __wbg_has_47b8eb16db92789a: function(arg0, arg1, arg2) {
                const ret = arg0.has(getStringFromWasm0(arg1, arg2));
                return ret;
            },
            __wbg_height_ac6f1c3f49c13fe2: function(arg0) {
                const ret = arg0.height;
                return ret;
            },
            __wbg_includes_40dae55ee37516e5: function(arg0, arg1, arg2) {
                const ret = arg0.includes(arg1, arg2);
                return ret;
            },
            __wbg_info_84c3d8389e807255: function(arg0) {
                const ret = arg0.info;
                return ret;
            },
            __wbg_instanceof_GpuOutOfMemoryError_72d4f143d9ec20cb: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof GPUOutOfMemoryError;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_GpuValidationError_cdb716bc3a7ed585: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof GPUValidationError;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_HtmlCanvasElement_89f280683e987b9e: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof HTMLCanvasElement;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_WebGl2RenderingContext_de6feb460369d6c6: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof WebGL2RenderingContext;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_instanceof_Window_5cf453f5639aa1a2: function(arg0) {
                let result;
                try {
                    result = arg0 instanceof Window;
                } catch (_) {
                    result = false;
                }
                const ret = result;
                return ret;
            },
            __wbg_invalidateFramebuffer_33206c632642a1bd: function() { return handleError(function (arg0, arg1, arg2) {
                arg0.invalidateFramebuffer(arg1 >>> 0, arg2);
            }, arguments); },
            __wbg_isFallbackAdapter_b70ad418e985d5d4: function(arg0) {
                const ret = arg0.isFallbackAdapter;
                return ret;
            },
            __wbg_is_2248ff2b575d60c7: function(arg0, arg1) {
                const ret = Object.is(arg0, arg1);
                return ret;
            },
            __wbg_label_b4e9fa173fb30ad5: function(arg0, arg1) {
                const ret = arg1.label;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
            },
            __wbg_length_186163212ffe79c3: function(arg0) {
                const ret = arg0.length;
                return ret;
            },
            __wbg_length_b03a0fabb109b1e9: function(arg0) {
                const ret = arg0.length;
                return ret;
            },
            __wbg_limits_06ac8ea738a0d93e: function(arg0) {
                const ret = arg0.limits;
                return ret;
            },
            __wbg_limits_7f838192fdb36f0a: function(arg0) {
                const ret = arg0.limits;
                return ret;
            },
            __wbg_linkProgram_99848b73f1ebab5f: function(arg0, arg1) {
                arg0.linkProgram(arg1);
            },
            __wbg_linkProgram_dee7cda98bacb12f: function(arg0, arg1) {
                arg0.linkProgram(arg1);
            },
            __wbg_mapAsync_84aeaa5780c9b8c7: function(arg0, arg1, arg2, arg3) {
                const ret = arg0.mapAsync(arg1 >>> 0, arg2, arg3);
                return ret;
            },
            __wbg_matchMedia_b53687d22ec43649: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.matchMedia(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            }, arguments); },
            __wbg_matches_a47c420646498c64: function(arg0) {
                const ret = arg0.matches;
                return ret;
            },
            __wbg_maxBindGroupsPlusVertexBuffers_5a483d2af94e57bc: function(arg0) {
                const ret = arg0.maxBindGroupsPlusVertexBuffers;
                return ret;
            },
            __wbg_maxBindGroups_8280e3693e832071: function(arg0) {
                const ret = arg0.maxBindGroups;
                return ret;
            },
            __wbg_maxBindingsPerBindGroup_872ae44e536b19a5: function(arg0) {
                const ret = arg0.maxBindingsPerBindGroup;
                return ret;
            },
            __wbg_maxBufferSize_9dae4d9428e42f10: function(arg0) {
                const ret = arg0.maxBufferSize;
                return ret;
            },
            __wbg_maxColorAttachmentBytesPerSample_12d8884108587e97: function(arg0) {
                const ret = arg0.maxColorAttachmentBytesPerSample;
                return ret;
            },
            __wbg_maxColorAttachments_deb9217288cb10c6: function(arg0) {
                const ret = arg0.maxColorAttachments;
                return ret;
            },
            __wbg_maxComputeInvocationsPerWorkgroup_c4b13a829ed0b259: function(arg0) {
                const ret = arg0.maxComputeInvocationsPerWorkgroup;
                return ret;
            },
            __wbg_maxComputeWorkgroupSizeX_91c35c8e026d4b2e: function(arg0) {
                const ret = arg0.maxComputeWorkgroupSizeX;
                return ret;
            },
            __wbg_maxComputeWorkgroupSizeY_961e4fb0ccb8e424: function(arg0) {
                const ret = arg0.maxComputeWorkgroupSizeY;
                return ret;
            },
            __wbg_maxComputeWorkgroupSizeZ_62c453c3b2746fe1: function(arg0) {
                const ret = arg0.maxComputeWorkgroupSizeZ;
                return ret;
            },
            __wbg_maxComputeWorkgroupStorageSize_dcf9113c2281aa6c: function(arg0) {
                const ret = arg0.maxComputeWorkgroupStorageSize;
                return ret;
            },
            __wbg_maxComputeWorkgroupsPerDimension_c8e2df553b22565e: function(arg0) {
                const ret = arg0.maxComputeWorkgroupsPerDimension;
                return ret;
            },
            __wbg_maxDynamicStorageBuffersPerPipelineLayout_43810e253385b7f5: function(arg0) {
                const ret = arg0.maxDynamicStorageBuffersPerPipelineLayout;
                return ret;
            },
            __wbg_maxDynamicUniformBuffersPerPipelineLayout_e160bf315957a3a1: function(arg0) {
                const ret = arg0.maxDynamicUniformBuffersPerPipelineLayout;
                return ret;
            },
            __wbg_maxInterStageShaderVariables_62a6f8974aa80170: function(arg0) {
                const ret = arg0.maxInterStageShaderVariables;
                return ret;
            },
            __wbg_maxSampledTexturesPerShaderStage_45931d0d0e425368: function(arg0) {
                const ret = arg0.maxSampledTexturesPerShaderStage;
                return ret;
            },
            __wbg_maxSamplersPerShaderStage_fd87b1b55a1bce03: function(arg0) {
                const ret = arg0.maxSamplersPerShaderStage;
                return ret;
            },
            __wbg_maxStorageBufferBindingSize_bec702c6fccfd8d5: function(arg0) {
                const ret = arg0.maxStorageBufferBindingSize;
                return ret;
            },
            __wbg_maxStorageBuffersPerShaderStage_36af4d1cf6b73e49: function(arg0) {
                const ret = arg0.maxStorageBuffersPerShaderStage;
                return ret;
            },
            __wbg_maxStorageTexturesPerShaderStage_bf0523882733deac: function(arg0) {
                const ret = arg0.maxStorageTexturesPerShaderStage;
                return ret;
            },
            __wbg_maxTextureArrayLayers_7c675fa8f48e7e97: function(arg0) {
                const ret = arg0.maxTextureArrayLayers;
                return ret;
            },
            __wbg_maxTextureDimension1D_bb69683afe97bf80: function(arg0) {
                const ret = arg0.maxTextureDimension1D;
                return ret;
            },
            __wbg_maxTextureDimension2D_cee54c433bf3b9c1: function(arg0) {
                const ret = arg0.maxTextureDimension2D;
                return ret;
            },
            __wbg_maxTextureDimension3D_ae26b58b1450c99a: function(arg0) {
                const ret = arg0.maxTextureDimension3D;
                return ret;
            },
            __wbg_maxUniformBufferBindingSize_00f780f64f0dd7c4: function(arg0) {
                const ret = arg0.maxUniformBufferBindingSize;
                return ret;
            },
            __wbg_maxUniformBuffersPerShaderStage_2c2054e9d96df7ad: function(arg0) {
                const ret = arg0.maxUniformBuffersPerShaderStage;
                return ret;
            },
            __wbg_maxVertexAttributes_425aa710b80da882: function(arg0) {
                const ret = arg0.maxVertexAttributes;
                return ret;
            },
            __wbg_maxVertexBufferArrayStride_8f0c56e46eba7c97: function(arg0) {
                const ret = arg0.maxVertexBufferArrayStride;
                return ret;
            },
            __wbg_maxVertexBuffers_473f2615aa5963dd: function(arg0) {
                const ret = arg0.maxVertexBuffers;
                return ret;
            },
            __wbg_message_bb4d389df9355dd2: function(arg0, arg1) {
                const ret = arg1.message;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
            },
            __wbg_minStorageBufferOffsetAlignment_f9ce2fe03a21a2e4: function(arg0) {
                const ret = arg0.minStorageBufferOffsetAlignment;
                return ret;
            },
            __wbg_minUniformBufferOffsetAlignment_bca4b75f696bbf11: function(arg0) {
                const ret = arg0.minUniformBufferOffsetAlignment;
                return ret;
            },
            __wbg_navigator_9641ab32fa612dd1: function(arg0) {
                const ret = arg0.navigator;
                return ret;
            },
            __wbg_navigator_c17f443a6fe5d38d: function(arg0) {
                const ret = arg0.navigator;
                return ret;
            },
            __wbg_new_2bfd0f45e60518b1: function() { return handleError(function (arg0, arg1) {
                const ret = new OffscreenCanvas(arg0 >>> 0, arg1 >>> 0);
                return ret;
            }, arguments); },
            __wbg_new_37eb45aaf5eeb820: function() {
                const ret = new Array();
                return ret;
            },
            __wbg_new_bb74001155cb8ca8: function() {
                const ret = new Error();
                return ret;
            },
            __wbg_new_cd4348290afae586: function() {
                const ret = new Object();
                return ret;
            },
            __wbg_new_typed_6b60b99917da27d6: function() {
                const ret = new Object();
                return ret;
            },
            __wbg_new_typed_e09da9dae6ee5935: function(arg0, arg1) {
                try {
                    var state0 = {a: arg0, b: arg1};
                    var cb0 = (arg0, arg1) => {
                        const a = state0.a;
                        state0.a = 0;
                        try {
                            return wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___js_sys_b1c51206aec32abd___Function_fn_wasm_bindgen_13ee6a147c6270a5___JsValue_____wasm_bindgen_13ee6a147c6270a5___sys__Undefined___js_sys_b1c51206aec32abd___Function_fn_wasm_bindgen_13ee6a147c6270a5___JsValue_____wasm_bindgen_13ee6a147c6270a5___sys__Undefined_______true_(a, state0.b, arg0, arg1);
                        } finally {
                            state0.a = a;
                        }
                    };
                    const ret = new Promise(cb0);
                    return ret;
                } finally {
                    state0.a = 0;
                }
            },
            __wbg_new_with_byte_offset_and_length_3cd32055a417f095: function(arg0, arg1, arg2) {
                const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
                return ret;
            },
            __wbg_now_043d245551b22f9c: function(arg0) {
                const ret = arg0.now();
                return ret;
            },
            __wbg_of_027eb15371aac932: function(arg0) {
                const ret = Array.of(arg0);
                return ret;
            },
            __wbg_onSubmittedWorkDone_f62945abb48d8a84: function(arg0) {
                const ret = arg0.onSubmittedWorkDone();
                return ret;
            },
            __wbg_performance_50e6a3f8dc80bbca: function(arg0) {
                const ret = arg0.performance;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_pixelStorei_43dc7945a0d4d49c: function(arg0, arg1, arg2) {
                arg0.pixelStorei(arg1 >>> 0, arg2);
            },
            __wbg_pixelStorei_6dd56b4da047c480: function(arg0, arg1, arg2) {
                arg0.pixelStorei(arg1 >>> 0, arg2);
            },
            __wbg_polygonOffset_41bc1f487e5f53fe: function(arg0, arg1, arg2) {
                arg0.polygonOffset(arg1, arg2);
            },
            __wbg_polygonOffset_acf166e60739dbfa: function(arg0, arg1, arg2) {
                arg0.polygonOffset(arg1, arg2);
            },
            __wbg_prototypesetcall_13162965d1fe7aa3: function(arg0, arg1, arg2) {
                Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
            },
            __wbg_push_c9f4893fc209f768: function(arg0, arg1) {
                const ret = arg0.push(arg1);
                return ret;
            },
            __wbg_queryCounterEXT_d43bae4582de7efd: function(arg0, arg1, arg2) {
                arg0.queryCounterEXT(arg1, arg2 >>> 0);
            },
            __wbg_querySelectorAll_6c1bd81d448fb480: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.querySelectorAll(getStringFromWasm0(arg1, arg2));
                return ret;
            }, arguments); },
            __wbg_querySelector_72201b7c62dc196b: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = arg0.querySelector(getStringFromWasm0(arg1, arg2));
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            }, arguments); },
            __wbg_queueMicrotask_1e8503a8a690f0a3: function(arg0) {
                const ret = arg0.queueMicrotask;
                return ret;
            },
            __wbg_queueMicrotask_4a1a4d8dd332373b: function(arg0) {
                queueMicrotask(arg0);
            },
            __wbg_queue_41b76331de876d9d: function(arg0) {
                const ret = arg0.queue;
                return ret;
            },
            __wbg_readBuffer_49d4b278b9838d40: function(arg0, arg1) {
                arg0.readBuffer(arg1 >>> 0);
            },
            __wbg_readPixels_22316d250d820aff: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
                arg0.readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
            }, arguments); },
            __wbg_readPixels_56a96c4d106afcbc: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
                arg0.readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
            }, arguments); },
            __wbg_readPixels_7c7b54a733248182: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
                arg0.readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
            }, arguments); },
            __wbg_renderbufferStorageMultisample_d5b01825cc08514b: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                arg0.renderbufferStorageMultisample(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
            },
            __wbg_renderbufferStorage_0d080ee86520d3bd: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.renderbufferStorage(arg1 >>> 0, arg2 >>> 0, arg3, arg4);
            },
            __wbg_renderbufferStorage_ebe429f577dae949: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.renderbufferStorage(arg1 >>> 0, arg2 >>> 0, arg3, arg4);
            },
            __wbg_requestAdapter_2cb1006baf7abd60: function(arg0) {
                const ret = arg0.requestAdapter();
                return ret;
            },
            __wbg_requestAdapter_b725cba92e32c4ce: function(arg0, arg1) {
                const ret = arg0.requestAdapter(arg1);
                return ret;
            },
            __wbg_requestDevice_45bc1149c3b98eaf: function(arg0, arg1) {
                const ret = arg0.requestDevice(arg1);
                return ret;
            },
            __wbg_resolve_dbf78ab4942478b0: function(arg0) {
                const ret = Promise.resolve(arg0);
                return ret;
            },
            __wbg_samplerParameterf_1fb6c6796e22f10f: function(arg0, arg1, arg2, arg3) {
                arg0.samplerParameterf(arg1, arg2 >>> 0, arg3);
            },
            __wbg_samplerParameteri_defcb3b230e696e7: function(arg0, arg1, arg2, arg3) {
                arg0.samplerParameteri(arg1, arg2 >>> 0, arg3);
            },
            __wbg_scissor_17956d9d23dfe025: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.scissor(arg1, arg2, arg3, arg4);
            },
            __wbg_scissor_bb7201f050f220ac: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.scissor(arg1, arg2, arg3, arg4);
            },
            __wbg_setBindGroup_09aa10258bd4dc11: function(arg0, arg1, arg2) {
                arg0.setBindGroup(arg1 >>> 0, arg2);
            },
            __wbg_setBindGroup_0e4a7b8b254673b9: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
                arg0.setBindGroup(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
            }, arguments); },
            __wbg_setBindGroup_1a145239764ea257: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
                arg0.setBindGroup(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
            }, arguments); },
            __wbg_setBindGroup_c3a39fd8d0149600: function(arg0, arg1, arg2) {
                arg0.setBindGroup(arg1 >>> 0, arg2);
            },
            __wbg_setPipeline_1368374122476dc8: function(arg0, arg1) {
                arg0.setPipeline(arg1);
            },
            __wbg_setPipeline_823fe7d48236b840: function(arg0, arg1) {
                arg0.setPipeline(arg1);
            },
            __wbg_setScissorRect_5b13089a95f2dadd: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.setScissorRect(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
            },
            __wbg_setVertexBuffer_6cb9f5432f1462e2: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3, arg4);
            },
            __wbg_setVertexBuffer_ff00b7e9bfd9257d: function(arg0, arg1, arg2, arg3) {
                arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3);
            },
            __wbg_set_118fefd528ab0a79: function(arg0, arg1, arg2) {
                arg0.set(arg1, arg2 >>> 0);
            },
            __wbg_set_2793479888329cb6: function() { return handleError(function (arg0, arg1, arg2) {
                const ret = Reflect.set(arg0, arg1, arg2);
                return ret;
            }, arguments); },
            __wbg_set_a_b8a7f56fa358e664: function(arg0, arg1) {
                arg0.a = arg1;
            },
            __wbg_set_access_46b3e62b6a30d608: function(arg0, arg1) {
                arg0.access = __wbindgen_enum_GpuStorageTextureAccess[arg1];
            },
            __wbg_set_address_mode_u_f67593eba8917d58: function(arg0, arg1) {
                arg0.addressModeU = __wbindgen_enum_GpuAddressMode[arg1];
            },
            __wbg_set_address_mode_v_8366e992c3ec2680: function(arg0, arg1) {
                arg0.addressModeV = __wbindgen_enum_GpuAddressMode[arg1];
            },
            __wbg_set_address_mode_w_53ae7ff9fc27d31b: function(arg0, arg1) {
                arg0.addressModeW = __wbindgen_enum_GpuAddressMode[arg1];
            },
            __wbg_set_alpha_041ea339c691473c: function(arg0, arg1) {
                arg0.alpha = arg1;
            },
            __wbg_set_alpha_mode_edf7c742eee7ac74: function(arg0, arg1) {
                arg0.alphaMode = __wbindgen_enum_GpuCanvasAlphaMode[arg1];
            },
            __wbg_set_alpha_to_coverage_enabled_66b16e2395f2145a: function(arg0, arg1) {
                arg0.alphaToCoverageEnabled = arg1 !== 0;
            },
            __wbg_set_array_layer_count_f652f75c2d85d00b: function(arg0, arg1) {
                arg0.arrayLayerCount = arg1 >>> 0;
            },
            __wbg_set_array_stride_f64_d4a555ef4287a90a: function(arg0, arg1) {
                arg0.arrayStride = arg1;
            },
            __wbg_set_aspect_0820f36ab69e69c7: function(arg0, arg1) {
                arg0.aspect = __wbindgen_enum_GpuTextureAspect[arg1];
            },
            __wbg_set_aspect_4eaf6e4f871485aa: function(arg0, arg1) {
                arg0.aspect = __wbindgen_enum_GpuTextureAspect[arg1];
            },
            __wbg_set_attributes_6753fa3eefb33349: function(arg0, arg1, arg2) {
                arg0.attributes = getArrayJsValueViewFromWasm0(arg1, arg2);
            },
            __wbg_set_b_1f0c9c5f62003b93: function(arg0, arg1) {
                arg0.b = arg1;
            },
            __wbg_set_base_array_layer_766baefe617e7450: function(arg0, arg1) {
                arg0.baseArrayLayer = arg1 >>> 0;
            },
            __wbg_set_base_mip_level_6c4e57453fe07edf: function(arg0, arg1) {
                arg0.baseMipLevel = arg1 >>> 0;
            },
            __wbg_set_beginning_of_pass_write_index_bda54c444871cf36: function(arg0, arg1) {
                arg0.beginningOfPassWriteIndex = arg1 >>> 0;
            },
            __wbg_set_beginning_of_pass_write_index_ce26d46a975d94e3: function(arg0, arg1) {
                arg0.beginningOfPassWriteIndex = arg1 >>> 0;
            },
            __wbg_set_bind_group_layouts_78efad321b8db306: function(arg0, arg1, arg2) {
                arg0.bindGroupLayouts = getArrayJsValueViewFromWasm0(arg1, arg2);
            },
            __wbg_set_binding_49121810d92bb656: function(arg0, arg1) {
                arg0.binding = arg1 >>> 0;
            },
            __wbg_set_binding_921af47b178839b7: function(arg0, arg1) {
                arg0.binding = arg1 >>> 0;
            },
            __wbg_set_blend_9b68fe4192551c11: function(arg0, arg1) {
                arg0.blend = arg1;
            },
            __wbg_set_buffer_e254d16772ccdf1f: function(arg0, arg1) {
                arg0.buffer = arg1;
            },
            __wbg_set_buffer_f0412de54dad59a0: function(arg0, arg1) {
                arg0.buffer = arg1;
            },
            __wbg_set_buffers_4bae4e561a9fa5bc: function(arg0, arg1, arg2) {
                arg0.buffers = getArrayJsValueViewFromWasm0(arg1, arg2);
            },
            __wbg_set_bytes_per_row_2edc74cba6ac1b69: function(arg0, arg1) {
                arg0.bytesPerRow = arg1 >>> 0;
            },
            __wbg_set_clear_value_gpu_color_dict_b12131c44b7f5a0b: function(arg0, arg1) {
                arg0.clearValue = arg1;
            },
            __wbg_set_code_71b4a924b594417d: function(arg0, arg1, arg2) {
                arg0.code = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_color_173beb3f62e0fe38: function(arg0, arg1) {
                arg0.color = arg1;
            },
            __wbg_set_color_attachments_bb683d934e2d6222: function(arg0, arg1, arg2) {
                arg0.colorAttachments = getArrayJsValueViewFromWasm0(arg1, arg2);
            },
            __wbg_set_compare_1f37d6bc9b4479b5: function(arg0, arg1) {
                arg0.compare = __wbindgen_enum_GpuCompareFunction[arg1];
            },
            __wbg_set_compare_9f209c2a21e5f8cc: function(arg0, arg1) {
                arg0.compare = __wbindgen_enum_GpuCompareFunction[arg1];
            },
            __wbg_set_compute_6328b3ac805fa957: function(arg0, arg1) {
                arg0.compute = arg1;
            },
            __wbg_set_count_02820c919f8591db: function(arg0, arg1) {
                arg0.count = arg1 >>> 0;
            },
            __wbg_set_cull_mode_2571e6256445d311: function(arg0, arg1) {
                arg0.cullMode = __wbindgen_enum_GpuCullMode[arg1];
            },
            __wbg_set_depth_bias_9ff79c074fab3395: function(arg0, arg1) {
                arg0.depthBias = arg1;
            },
            __wbg_set_depth_bias_clamp_b9dfc0449214a137: function(arg0, arg1) {
                arg0.depthBiasClamp = arg1;
            },
            __wbg_set_depth_bias_slope_scale_bacfbadca941fcf5: function(arg0, arg1) {
                arg0.depthBiasSlopeScale = arg1;
            },
            __wbg_set_depth_clear_value_bbfc087b5b4b6d06: function(arg0, arg1) {
                arg0.depthClearValue = arg1;
            },
            __wbg_set_depth_compare_706f99d5a7658960: function(arg0, arg1) {
                arg0.depthCompare = __wbindgen_enum_GpuCompareFunction[arg1];
            },
            __wbg_set_depth_fail_op_8cc03f38206b087f: function(arg0, arg1) {
                arg0.depthFailOp = __wbindgen_enum_GpuStencilOperation[arg1];
            },
            __wbg_set_depth_load_op_96d40a8e26164544: function(arg0, arg1) {
                arg0.depthLoadOp = __wbindgen_enum_GpuLoadOp[arg1];
            },
            __wbg_set_depth_or_array_layers_41d9bf5003084813: function(arg0, arg1) {
                arg0.depthOrArrayLayers = arg1 >>> 0;
            },
            __wbg_set_depth_read_only_8be6458323119975: function(arg0, arg1) {
                arg0.depthReadOnly = arg1 !== 0;
            },
            __wbg_set_depth_stencil_attachment_7a083fe21730d473: function(arg0, arg1) {
                arg0.depthStencilAttachment = arg1;
            },
            __wbg_set_depth_stencil_f138ba545e6cc4d4: function(arg0, arg1) {
                arg0.depthStencil = arg1;
            },
            __wbg_set_depth_store_op_3e7ee4bf89cda835: function(arg0, arg1) {
                arg0.depthStoreOp = __wbindgen_enum_GpuStoreOp[arg1];
            },
            __wbg_set_depth_write_enabled_c1ad58fe7207ef11: function(arg0, arg1) {
                arg0.depthWriteEnabled = arg1 !== 0;
            },
            __wbg_set_device_2b7e0e4f69784351: function(arg0, arg1) {
                arg0.device = arg1;
            },
            __wbg_set_dimension_536f245d466cca10: function(arg0, arg1) {
                arg0.dimension = __wbindgen_enum_GpuTextureDimension[arg1];
            },
            __wbg_set_dimension_91e724d498a35c9a: function(arg0, arg1) {
                arg0.dimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
            },
            __wbg_set_dst_factor_9789f02ad2a0c61e: function(arg0, arg1) {
                arg0.dstFactor = __wbindgen_enum_GpuBlendFactor[arg1];
            },
            __wbg_set_end_of_pass_write_index_4b0931cfb0c1f44c: function(arg0, arg1) {
                arg0.endOfPassWriteIndex = arg1 >>> 0;
            },
            __wbg_set_end_of_pass_write_index_bd996c6cc50b794b: function(arg0, arg1) {
                arg0.endOfPassWriteIndex = arg1 >>> 0;
            },
            __wbg_set_entries_02ae66da25c9ddae: function(arg0, arg1, arg2) {
                arg0.entries = getArrayJsValueViewFromWasm0(arg1, arg2);
            },
            __wbg_set_entries_8036374af27741dc: function(arg0, arg1, arg2) {
                arg0.entries = getArrayJsValueViewFromWasm0(arg1, arg2);
            },
            __wbg_set_entry_point_79bb4d8fb01c97ec: function(arg0, arg1, arg2) {
                arg0.entryPoint = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_entry_point_89ecd5ddd9c56d5c: function(arg0, arg1, arg2) {
                arg0.entryPoint = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_entry_point_b4eb7d2dd59b5c76: function(arg0, arg1, arg2) {
                arg0.entryPoint = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_external_texture_34c43368e77cc844: function(arg0, arg1) {
                arg0.externalTexture = arg1;
            },
            __wbg_set_fail_op_e901d7b520ae0e15: function(arg0, arg1) {
                arg0.failOp = __wbindgen_enum_GpuStencilOperation[arg1];
            },
            __wbg_set_format_1d70746424721ed1: function(arg0, arg1) {
                arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
            },
            __wbg_set_format_594ffafc6477f59d: function(arg0, arg1) {
                arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
            },
            __wbg_set_format_781ab1cf2f8f2465: function(arg0, arg1) {
                arg0.format = __wbindgen_enum_GpuVertexFormat[arg1];
            },
            __wbg_set_format_8003fc2174c14359: function(arg0, arg1) {
                arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
            },
            __wbg_set_format_c1603ca98ab9cb13: function(arg0, arg1) {
                arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
            },
            __wbg_set_format_cdccd8422de3c885: function(arg0, arg1) {
                arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
            },
            __wbg_set_format_df8baca64e04f5ef: function(arg0, arg1) {
                arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
            },
            __wbg_set_fragment_3635fd557dbd99bc: function(arg0, arg1) {
                arg0.fragment = arg1;
            },
            __wbg_set_front_face_13105806890daab7: function(arg0, arg1) {
                arg0.frontFace = __wbindgen_enum_GpuFrontFace[arg1];
            },
            __wbg_set_g_9d2890d6fb93dba4: function(arg0, arg1) {
                arg0.g = arg1;
            },
            __wbg_set_has_dynamic_offset_0086bc7faede16d1: function(arg0, arg1) {
                arg0.hasDynamicOffset = arg1 !== 0;
            },
            __wbg_set_height_546367a98b9ab695: function(arg0, arg1) {
                arg0.height = arg1 >>> 0;
            },
            __wbg_set_height_ae0610d318c3428c: function(arg0, arg1) {
                arg0.height = arg1 >>> 0;
            },
            __wbg_set_height_ae24c2f7d580ae0a: function(arg0, arg1) {
                arg0.height = arg1 >>> 0;
            },
            __wbg_set_label_16858d210dffe9ff: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_1d7c053bc726e08f: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_1e431973068c0ea4: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_2d3e4456ff9a120a: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_39038ba992db5f45: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_5e1a79b630ee9fa3: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_89172d31c03b3375: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_a71d5f5c41bed969: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_d133ac18919ae10f: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_dcaa5d479cbb5311: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_dcb16ec336a5420d: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_e9f1af9890ad68b6: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_ea59541fa72ee985: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_ef8bfd8864e594a4: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_label_f9171cd5c971dbf2: function(arg0, arg1, arg2) {
                arg0.label = getStringFromWasm0(arg1, arg2);
            },
            __wbg_set_layout_6b757fab7265d0f7: function(arg0, arg1) {
                arg0.layout = arg1;
            },
            __wbg_set_layout_905339a584186697: function(arg0, arg1) {
                arg0.layout = arg1;
            },
            __wbg_set_layout_bfddc4788acb0736: function(arg0, arg1) {
                arg0.layout = arg1;
            },
            __wbg_set_layout_gpu_auto_layout_mode_0e86cf27fd7c9e0c: function(arg0, arg1) {
                arg0.layout = __wbindgen_enum_GpuAutoLayoutMode[arg1];
            },
            __wbg_set_layout_gpu_auto_layout_mode_89827b74212f80b4: function(arg0, arg1) {
                arg0.layout = __wbindgen_enum_GpuAutoLayoutMode[arg1];
            },
            __wbg_set_load_op_9a24b40a8df7115b: function(arg0, arg1) {
                arg0.loadOp = __wbindgen_enum_GpuLoadOp[arg1];
            },
            __wbg_set_lod_max_clamp_c1935a38a79d7f3b: function(arg0, arg1) {
                arg0.lodMaxClamp = arg1;
            },
            __wbg_set_lod_min_clamp_27d01e7c2938fe09: function(arg0, arg1) {
                arg0.lodMinClamp = arg1;
            },
            __wbg_set_mag_filter_657599d734cdc0d9: function(arg0, arg1) {
                arg0.magFilter = __wbindgen_enum_GpuFilterMode[arg1];
            },
            __wbg_set_mapped_at_creation_a4cd55019116b792: function(arg0, arg1) {
                arg0.mappedAtCreation = arg1 !== 0;
            },
            __wbg_set_mask_7a39674fcbc00286: function(arg0, arg1) {
                arg0.mask = arg1 >>> 0;
            },
            __wbg_set_max_anisotropy_b05f95dcdfdae8bb: function(arg0, arg1) {
                arg0.maxAnisotropy = arg1;
            },
            __wbg_set_min_binding_size_f64_2458ad9bffad7ea7: function(arg0, arg1) {
                arg0.minBindingSize = arg1;
            },
            __wbg_set_min_filter_93e33f778b66ecb0: function(arg0, arg1) {
                arg0.minFilter = __wbindgen_enum_GpuFilterMode[arg1];
            },
            __wbg_set_mip_level_count_19560e4f63320832: function(arg0, arg1) {
                arg0.mipLevelCount = arg1 >>> 0;
            },
            __wbg_set_mip_level_count_ef313d3a441aa51e: function(arg0, arg1) {
                arg0.mipLevelCount = arg1 >>> 0;
            },
            __wbg_set_mip_level_dbdf4ee3ba07c689: function(arg0, arg1) {
                arg0.mipLevel = arg1 >>> 0;
            },
            __wbg_set_mipmap_filter_aacb60d49a794ef5: function(arg0, arg1) {
                arg0.mipmapFilter = __wbindgen_enum_GpuMipmapFilterMode[arg1];
            },
            __wbg_set_mode_73d9eb9243253108: function(arg0, arg1) {
                arg0.mode = __wbindgen_enum_GpuCanvasToneMappingMode[arg1];
            },
            __wbg_set_module_85e1d5368a2b9422: function(arg0, arg1) {
                arg0.module = arg1;
            },
            __wbg_set_module_bffb3e27ccc03d8d: function(arg0, arg1) {
                arg0.module = arg1;
            },
            __wbg_set_module_c746beebaa6ede01: function(arg0, arg1) {
                arg0.module = arg1;
            },
            __wbg_set_multisample_7471ab77b8e5a567: function(arg0, arg1) {
                arg0.multisample = arg1;
            },
            __wbg_set_multisampled_d385dcc24cbcdac9: function(arg0, arg1) {
                arg0.multisampled = arg1 !== 0;
            },
            __wbg_set_offset_f64_106ac70d42ed0b22: function(arg0, arg1) {
                arg0.offset = arg1;
            },
            __wbg_set_offset_f64_b1157147188cb175: function(arg0, arg1) {
                arg0.offset = arg1;
            },
            __wbg_set_offset_f64_f65aa794a52b87dc: function(arg0, arg1) {
                arg0.offset = arg1;
            },
            __wbg_set_onuncapturederror_1a772a1feba5ec1d: function(arg0, arg1) {
                arg0.onuncapturederror = arg1;
            },
            __wbg_set_operation_b82c9485304b393c: function(arg0, arg1) {
                arg0.operation = __wbindgen_enum_GpuBlendOperation[arg1];
            },
            __wbg_set_origin_gpu_origin_3d_dict_c2b5dc3bd3763139: function(arg0, arg1) {
                arg0.origin = arg1;
            },
            __wbg_set_pass_op_3b1d192c54b1a827: function(arg0, arg1) {
                arg0.passOp = __wbindgen_enum_GpuStencilOperation[arg1];
            },
            __wbg_set_power_preference_06107ba643e66249: function(arg0, arg1) {
                arg0.powerPreference = __wbindgen_enum_GpuPowerPreference[arg1];
            },
            __wbg_set_primitive_14377cb8cbd23584: function(arg0, arg1) {
                arg0.primitive = arg1;
            },
            __wbg_set_query_set_a1af7e0b244eb437: function(arg0, arg1) {
                arg0.querySet = arg1;
            },
            __wbg_set_query_set_da20f7d5b37c78e1: function(arg0, arg1) {
                arg0.querySet = arg1;
            },
            __wbg_set_r_473d4bd5f6ddbe44: function(arg0, arg1) {
                arg0.r = arg1;
            },
            __wbg_set_required_features_ef8aeb4d5f909e03: function(arg0, arg1, arg2) {
                arg0.requiredFeatures = getArrayJsValueViewFromWasm0(arg1, arg2);
            },
            __wbg_set_required_limits_a80a53fd4744ccb2: function(arg0, arg1) {
                arg0.requiredLimits = arg1;
            },
            __wbg_set_resolve_target_gpu_texture_view_6c8f66cd96330a45: function(arg0, arg1) {
                arg0.resolveTarget = arg1;
            },
            __wbg_set_resource_d37005b6a3d3375f: function(arg0, arg1) {
                arg0.resource = arg1;
            },
            __wbg_set_resource_gpu_buffer_binding_8e9be21bc9a45330: function(arg0, arg1) {
                arg0.resource = arg1;
            },
            __wbg_set_resource_gpu_texture_view_e3bf3810e7168d2b: function(arg0, arg1) {
                arg0.resource = arg1;
            },
            __wbg_set_rows_per_image_e9f4da51b05f4ee1: function(arg0, arg1) {
                arg0.rowsPerImage = arg1 >>> 0;
            },
            __wbg_set_sample_count_a4d54cf5bcf23914: function(arg0, arg1) {
                arg0.sampleCount = arg1 >>> 0;
            },
            __wbg_set_sample_type_660a4d77497517e2: function(arg0, arg1) {
                arg0.sampleType = __wbindgen_enum_GpuTextureSampleType[arg1];
            },
            __wbg_set_sampler_35b5b2e68532815b: function(arg0, arg1) {
                arg0.sampler = arg1;
            },
            __wbg_set_shader_location_e0fdb186c91ac5df: function(arg0, arg1) {
                arg0.shaderLocation = arg1 >>> 0;
            },
            __wbg_set_size_f64_04b949f2af8176ba: function(arg0, arg1) {
                arg0.size = arg1;
            },
            __wbg_set_size_f64_2418f77508fb05d7: function(arg0, arg1) {
                arg0.size = arg1;
            },
            __wbg_set_size_gpu_extent_3d_dict_7180918365da46d5: function(arg0, arg1) {
                arg0.size = arg1;
            },
            __wbg_set_src_factor_e72cbac8c7084adc: function(arg0, arg1) {
                arg0.srcFactor = __wbindgen_enum_GpuBlendFactor[arg1];
            },
            __wbg_set_stencil_back_4df76dba55137aa9: function(arg0, arg1) {
                arg0.stencilBack = arg1;
            },
            __wbg_set_stencil_clear_value_686c18ee265a672c: function(arg0, arg1) {
                arg0.stencilClearValue = arg1 >>> 0;
            },
            __wbg_set_stencil_front_ec8c0eedf46d4973: function(arg0, arg1) {
                arg0.stencilFront = arg1;
            },
            __wbg_set_stencil_load_op_1bce6d97f680eda7: function(arg0, arg1) {
                arg0.stencilLoadOp = __wbindgen_enum_GpuLoadOp[arg1];
            },
            __wbg_set_stencil_read_mask_1f242f58d95e02e1: function(arg0, arg1) {
                arg0.stencilReadMask = arg1 >>> 0;
            },
            __wbg_set_stencil_read_only_50c3a3b3cbdc83c1: function(arg0, arg1) {
                arg0.stencilReadOnly = arg1 !== 0;
            },
            __wbg_set_stencil_store_op_9703f525bc5e8e0c: function(arg0, arg1) {
                arg0.stencilStoreOp = __wbindgen_enum_GpuStoreOp[arg1];
            },
            __wbg_set_stencil_write_mask_463f4b6208354e15: function(arg0, arg1) {
                arg0.stencilWriteMask = arg1 >>> 0;
            },
            __wbg_set_step_mode_c10a50b921560088: function(arg0, arg1) {
                arg0.stepMode = __wbindgen_enum_GpuVertexStepMode[arg1];
            },
            __wbg_set_storage_texture_c2ed1f3eb327112e: function(arg0, arg1) {
                arg0.storageTexture = arg1;
            },
            __wbg_set_store_op_e3edba4d13e36cbf: function(arg0, arg1) {
                arg0.storeOp = __wbindgen_enum_GpuStoreOp[arg1];
            },
            __wbg_set_strip_index_format_a7cb0b950f8653f8: function(arg0, arg1) {
                arg0.stripIndexFormat = __wbindgen_enum_GpuIndexFormat[arg1];
            },
            __wbg_set_targets_b24b23a20f1c7f7f: function(arg0, arg1, arg2) {
                arg0.targets = getArrayJsValueViewFromWasm0(arg1, arg2);
            },
            __wbg_set_texture_39ac35f54ef4c3ba: function(arg0, arg1) {
                arg0.texture = arg1;
            },
            __wbg_set_texture_c89ca659682f2fd6: function(arg0, arg1) {
                arg0.texture = arg1;
            },
            __wbg_set_timestamp_writes_90e77f617d9af260: function(arg0, arg1) {
                arg0.timestampWrites = arg1;
            },
            __wbg_set_timestamp_writes_9fd1c7b5cb0a6a5c: function(arg0, arg1) {
                arg0.timestampWrites = arg1;
            },
            __wbg_set_tone_mapping_92c0666fef742c68: function(arg0, arg1) {
                arg0.toneMapping = arg1;
            },
            __wbg_set_topology_ed2942658d3771ab: function(arg0, arg1) {
                arg0.topology = __wbindgen_enum_GpuPrimitiveTopology[arg1];
            },
            __wbg_set_type_6cc967b44400577b: function(arg0, arg1) {
                arg0.type = __wbindgen_enum_GpuSamplerBindingType[arg1];
            },
            __wbg_set_type_dd6c71bc279b420e: function(arg0, arg1) {
                arg0.type = __wbindgen_enum_GpuBufferBindingType[arg1];
            },
            __wbg_set_unclipped_depth_f24953dc0eb8c84c: function(arg0, arg1) {
                arg0.unclippedDepth = arg1 !== 0;
            },
            __wbg_set_usage_1d663151adcb7f1a: function(arg0, arg1) {
                arg0.usage = arg1 >>> 0;
            },
            __wbg_set_usage_63cd5e88050b26e8: function(arg0, arg1) {
                arg0.usage = arg1 >>> 0;
            },
            __wbg_set_usage_6c4bd5f209a0579c: function(arg0, arg1) {
                arg0.usage = arg1 >>> 0;
            },
            __wbg_set_usage_a65753b69e218ec9: function(arg0, arg1) {
                arg0.usage = arg1 >>> 0;
            },
            __wbg_set_vertex_b8cfd3671e5edfbb: function(arg0, arg1) {
                arg0.vertex = arg1;
            },
            __wbg_set_view_dimension_ce6e77a9eb67aa53: function(arg0, arg1) {
                arg0.viewDimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
            },
            __wbg_set_view_dimension_d5c87d8c6f1266b5: function(arg0, arg1) {
                arg0.viewDimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
            },
            __wbg_set_view_formats_8b40d61be4d5be29: function(arg0, arg1, arg2) {
                arg0.viewFormats = getArrayJsValueViewFromWasm0(arg1, arg2);
            },
            __wbg_set_view_formats_ea87e3d240842107: function(arg0, arg1, arg2) {
                arg0.viewFormats = getArrayJsValueViewFromWasm0(arg1, arg2);
            },
            __wbg_set_view_gpu_texture_view_bf636cc659953b65: function(arg0, arg1) {
                arg0.view = arg1;
            },
            __wbg_set_view_gpu_texture_view_d53664457085e78b: function(arg0, arg1) {
                arg0.view = arg1;
            },
            __wbg_set_visibility_f68d6576737b8b45: function(arg0, arg1) {
                arg0.visibility = arg1 >>> 0;
            },
            __wbg_set_width_5f89a551b5472887: function(arg0, arg1) {
                arg0.width = arg1 >>> 0;
            },
            __wbg_set_width_eb3001e1dcabb909: function(arg0, arg1) {
                arg0.width = arg1 >>> 0;
            },
            __wbg_set_width_fc76d04c0309b403: function(arg0, arg1) {
                arg0.width = arg1 >>> 0;
            },
            __wbg_set_write_mask_51378295d978a0f8: function(arg0, arg1) {
                arg0.writeMask = arg1 >>> 0;
            },
            __wbg_set_x_aa58610e8a2b3347: function(arg0, arg1) {
                arg0.x = arg1 >>> 0;
            },
            __wbg_set_y_6a3e943d5ad07e0d: function(arg0, arg1) {
                arg0.y = arg1 >>> 0;
            },
            __wbg_set_z_dc1a88e5f0e1508d: function(arg0, arg1) {
                arg0.z = arg1 >>> 0;
            },
            __wbg_shaderSource_519d0f56b613b2f9: function(arg0, arg1, arg2, arg3) {
                arg0.shaderSource(arg1, getStringFromWasm0(arg2, arg3));
            },
            __wbg_shaderSource_c4c06a3cbc2e7bf3: function(arg0, arg1, arg2, arg3) {
                arg0.shaderSource(arg1, getStringFromWasm0(arg2, arg3));
            },
            __wbg_stack_575c4e0918581b9c: function(arg0, arg1) {
                const ret = arg1.stack;
                const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
                const len1 = WASM_VECTOR_LEN;
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
                getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
            },
            __wbg_static_accessor_GLOBAL_THIS_003ee833af9699fc: function() {
                const ret = typeof globalThis === 'undefined' ? undefined : globalThis;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_static_accessor_GLOBAL_e4293fb22cbe77da: function() {
                const ret = typeof global === 'undefined' ? undefined : global;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_static_accessor_SELF_53bb20457401453d: function() {
                const ret = typeof self === 'undefined' ? undefined : self;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_static_accessor_WINDOW_b5e0810bfecb7feb: function() {
                const ret = typeof window === 'undefined' ? undefined : window;
                return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
            },
            __wbg_stencilFuncSeparate_bd578fbb298ad0e6: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.stencilFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3, arg4 >>> 0);
            },
            __wbg_stencilFuncSeparate_c4e19ab2f6a233b9: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.stencilFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3, arg4 >>> 0);
            },
            __wbg_stencilMaskSeparate_41c3f5591b508051: function(arg0, arg1, arg2) {
                arg0.stencilMaskSeparate(arg1 >>> 0, arg2 >>> 0);
            },
            __wbg_stencilMaskSeparate_b009d9ab29dc5926: function(arg0, arg1, arg2) {
                arg0.stencilMaskSeparate(arg1 >>> 0, arg2 >>> 0);
            },
            __wbg_stencilMask_1fe24adb96500c11: function(arg0, arg1) {
                arg0.stencilMask(arg1 >>> 0);
            },
            __wbg_stencilMask_6d5dbf91337de94e: function(arg0, arg1) {
                arg0.stencilMask(arg1 >>> 0);
            },
            __wbg_stencilOpSeparate_9a2068a0767d6507: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.stencilOpSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
            },
            __wbg_stencilOpSeparate_f4319497016868f8: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.stencilOpSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
            },
            __wbg_subgroupMaxSize_b33b2d37e3f1f5f5: function(arg0) {
                const ret = arg0.subgroupMaxSize;
                return ret;
            },
            __wbg_subgroupMinSize_ff381626432f1cbc: function(arg0) {
                const ret = arg0.subgroupMinSize;
                return ret;
            },
            __wbg_submit_c4379e221512d3fa: function(arg0, arg1, arg2) {
                arg0.submit(getArrayJsValueViewFromWasm0(arg1, arg2));
            },
            __wbg_texImage2D_20b5002326193fe6: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
                arg0.texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
            }, arguments); },
            __wbg_texImage2D_5048d2cd8540da16: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
                arg0.texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
            }, arguments); },
            __wbg_texImage2D_84bdef619a1ea935: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
                arg0.texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
            }, arguments); },
            __wbg_texImage3D_5b7fe3781e1db22a: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
                arg0.texImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8 >>> 0, arg9 >>> 0, arg10);
            }, arguments); },
            __wbg_texImage3D_abf2fdb0f8c4c1b6: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
                arg0.texImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8 >>> 0, arg9 >>> 0, arg10);
            }, arguments); },
            __wbg_texParameteri_0afc5d515132fed2: function(arg0, arg1, arg2, arg3) {
                arg0.texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
            },
            __wbg_texParameteri_e24e4fdc17153838: function(arg0, arg1, arg2, arg3) {
                arg0.texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
            },
            __wbg_texStorage2D_e0ebb5940fa1fc5e: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                arg0.texStorage2D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
            },
            __wbg_texStorage3D_a82e5be79cb278dc: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
                arg0.texStorage3D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5, arg6);
            },
            __wbg_texSubImage2D_0fed1a7412bc7a63: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
                arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
            }, arguments); },
            __wbg_texSubImage2D_1434c994889108aa: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
                arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
            }, arguments); },
            __wbg_texSubImage2D_4343af4633b8b8f8: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
                arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
            }, arguments); },
            __wbg_texSubImage2D_4aef07460a86c81e: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
                arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
            }, arguments); },
            __wbg_texSubImage2D_9c7e2d1250b1f95f: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
                arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
            }, arguments); },
            __wbg_texSubImage2D_a26a8700e734449a: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
                arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
            }, arguments); },
            __wbg_texSubImage2D_a271f112c3b60b18: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
                arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
            }, arguments); },
            __wbg_texSubImage2D_baff05c8a8de2e17: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
                arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
            }, arguments); },
            __wbg_texSubImage3D_00e908f23d33228d: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
                arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
            }, arguments); },
            __wbg_texSubImage3D_286489de7bf7239a: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
                arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
            }, arguments); },
            __wbg_texSubImage3D_32223c3dd7dcdd78: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
                arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
            }, arguments); },
            __wbg_texSubImage3D_5e8b27dc040faf8d: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
                arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
            }, arguments); },
            __wbg_texSubImage3D_b1b631c3011a209e: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
                arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
            }, arguments); },
            __wbg_texSubImage3D_e8f20815b70bc7cb: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
                arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
            }, arguments); },
            __wbg_texSubImage3D_ed8982daddcccd1e: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
                arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
            }, arguments); },
            __wbg_then_656f253a5dd7d8ea: function(arg0, arg1, arg2) {
                const ret = arg0.then(arg1, arg2);
                return ret;
            },
            __wbg_then_e3f7f2d365708be8: function(arg0, arg1) {
                const ret = arg0.then(arg1);
                return ret;
            },
            __wbg_unconfigure_7ca436ff70c0bcc3: function(arg0) {
                arg0.unconfigure();
            },
            __wbg_uniform1f_0b36dc15cb2b4991: function(arg0, arg1, arg2) {
                arg0.uniform1f(arg1, arg2);
            },
            __wbg_uniform1f_fe417acc7c7a1dc5: function(arg0, arg1, arg2) {
                arg0.uniform1f(arg1, arg2);
            },
            __wbg_uniform1i_385ebc488d4d763f: function(arg0, arg1, arg2) {
                arg0.uniform1i(arg1, arg2);
            },
            __wbg_uniform1i_778a02a48ecdceed: function(arg0, arg1, arg2) {
                arg0.uniform1i(arg1, arg2);
            },
            __wbg_uniform1ui_79071fccd0396b88: function(arg0, arg1, arg2) {
                arg0.uniform1ui(arg1, arg2 >>> 0);
            },
            __wbg_uniform2fv_486658e6bcddfbaf: function(arg0, arg1, arg2, arg3) {
                arg0.uniform2fv(arg1, getArrayF32FromWasm0(arg2, arg3));
            },
            __wbg_uniform2fv_71fe605c72879b61: function(arg0, arg1, arg2, arg3) {
                arg0.uniform2fv(arg1, getArrayF32FromWasm0(arg2, arg3));
            },
            __wbg_uniform2iv_80c1acefe7258756: function(arg0, arg1, arg2, arg3) {
                arg0.uniform2iv(arg1, getArrayI32FromWasm0(arg2, arg3));
            },
            __wbg_uniform2iv_d6efe574c69f5aed: function(arg0, arg1, arg2, arg3) {
                arg0.uniform2iv(arg1, getArrayI32FromWasm0(arg2, arg3));
            },
            __wbg_uniform2uiv_f9619772d6706c37: function(arg0, arg1, arg2, arg3) {
                arg0.uniform2uiv(arg1, getArrayU32FromWasm0(arg2, arg3));
            },
            __wbg_uniform3fv_431207a9407b7d29: function(arg0, arg1, arg2, arg3) {
                arg0.uniform3fv(arg1, getArrayF32FromWasm0(arg2, arg3));
            },
            __wbg_uniform3fv_58fa948c65cb5f6c: function(arg0, arg1, arg2, arg3) {
                arg0.uniform3fv(arg1, getArrayF32FromWasm0(arg2, arg3));
            },
            __wbg_uniform3iv_11889262cf74698c: function(arg0, arg1, arg2, arg3) {
                arg0.uniform3iv(arg1, getArrayI32FromWasm0(arg2, arg3));
            },
            __wbg_uniform3iv_e9d02a45a4149457: function(arg0, arg1, arg2, arg3) {
                arg0.uniform3iv(arg1, getArrayI32FromWasm0(arg2, arg3));
            },
            __wbg_uniform3uiv_b71a1165273e4af4: function(arg0, arg1, arg2, arg3) {
                arg0.uniform3uiv(arg1, getArrayU32FromWasm0(arg2, arg3));
            },
            __wbg_uniform4f_14a3638afe93aa8f: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                arg0.uniform4f(arg1, arg2, arg3, arg4, arg5);
            },
            __wbg_uniform4f_5c31fb6dab645ad8: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                arg0.uniform4f(arg1, arg2, arg3, arg4, arg5);
            },
            __wbg_uniform4fv_a1415c11925ac651: function(arg0, arg1, arg2, arg3) {
                arg0.uniform4fv(arg1, getArrayF32FromWasm0(arg2, arg3));
            },
            __wbg_uniform4fv_d3afea28181f5342: function(arg0, arg1, arg2, arg3) {
                arg0.uniform4fv(arg1, getArrayF32FromWasm0(arg2, arg3));
            },
            __wbg_uniform4iv_c9e8520342091b0a: function(arg0, arg1, arg2, arg3) {
                arg0.uniform4iv(arg1, getArrayI32FromWasm0(arg2, arg3));
            },
            __wbg_uniform4iv_fa18eabf499f4df7: function(arg0, arg1, arg2, arg3) {
                arg0.uniform4iv(arg1, getArrayI32FromWasm0(arg2, arg3));
            },
            __wbg_uniform4uiv_7a5b848aa1084b38: function(arg0, arg1, arg2, arg3) {
                arg0.uniform4uiv(arg1, getArrayU32FromWasm0(arg2, arg3));
            },
            __wbg_uniformBlockBinding_61c5a97721e0f36b: function(arg0, arg1, arg2, arg3) {
                arg0.uniformBlockBinding(arg1, arg2 >>> 0, arg3 >>> 0);
            },
            __wbg_uniformMatrix2fv_0a42d3c91d7cb169: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.uniformMatrix2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
            },
            __wbg_uniformMatrix2fv_8b3774cc1f2dd2af: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.uniformMatrix2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
            },
            __wbg_uniformMatrix2x3fv_f9e50997fac55ae7: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.uniformMatrix2x3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
            },
            __wbg_uniformMatrix2x4fv_080d208b4f9c72be: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.uniformMatrix2x4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
            },
            __wbg_uniformMatrix3fv_15b2b1f5505b8506: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.uniformMatrix3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
            },
            __wbg_uniformMatrix3fv_df1958f8b617c8d6: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.uniformMatrix3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
            },
            __wbg_uniformMatrix3x2fv_6730abc8ef13d7d3: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.uniformMatrix3x2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
            },
            __wbg_uniformMatrix3x4fv_3eb72e0d85856655: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.uniformMatrix3x4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
            },
            __wbg_uniformMatrix4fv_cd0ba72d1d780759: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.uniformMatrix4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
            },
            __wbg_uniformMatrix4fv_edd2b51dbb1caffd: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.uniformMatrix4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
            },
            __wbg_uniformMatrix4x2fv_b0c23910facd8e86: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.uniformMatrix4x2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
            },
            __wbg_uniformMatrix4x3fv_b6f9b241f392f245: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.uniformMatrix4x3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
            },
            __wbg_unmap_073211b910c6a369: function(arg0) {
                arg0.unmap();
            },
            __wbg_useProgram_d9c24a7d8632c4c1: function(arg0, arg1) {
                arg0.useProgram(arg1);
            },
            __wbg_useProgram_f6341ea2eeea1e5a: function(arg0, arg1) {
                arg0.useProgram(arg1);
            },
            __wbg_valueOf_2ad4a09774ac959e: function(arg0) {
                const ret = arg0.valueOf();
                return ret;
            },
            __wbg_vertexAttribDivisorANGLE_5bec9a774ccd1cf8: function(arg0, arg1, arg2) {
                arg0.vertexAttribDivisorANGLE(arg1 >>> 0, arg2 >>> 0);
            },
            __wbg_vertexAttribDivisor_5b64fc697c0496e1: function(arg0, arg1, arg2) {
                arg0.vertexAttribDivisor(arg1 >>> 0, arg2 >>> 0);
            },
            __wbg_vertexAttribIPointer_a6240b64263fb63b: function(arg0, arg1, arg2, arg3, arg4, arg5) {
                arg0.vertexAttribIPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
            },
            __wbg_vertexAttribPointer_39b706eb26c50057: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
                arg0.vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
            },
            __wbg_vertexAttribPointer_9e0e4517b8d274bd: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
                arg0.vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
            },
            __wbg_viewport_aaac6dd8cff1831a: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.viewport(arg1, arg2, arg3, arg4);
            },
            __wbg_viewport_cdae9e8b2f33343d: function(arg0, arg1, arg2, arg3, arg4) {
                arg0.viewport(arg1, arg2, arg3, arg4);
            },
            __wbg_warn_228ff2257384d295: function(arg0) {
                console.warn(arg0);
            },
            __wbg_width_babc5dc7654771b1: function(arg0) {
                const ret = arg0.width;
                return ret;
            },
            __wbg_writeBuffer_dbc00ede89a23b5e: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
                arg0.writeBuffer(arg1, arg2, getArrayU8FromWasm0(arg3, arg4), arg5, arg6);
            }, arguments); },
            __wbg_writeTexture_ddcddb10ef4a4155: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
                arg0.writeTexture(arg1, getArrayU8FromWasm0(arg2, arg3), arg4, arg5);
            }, arguments); },
            __wbindgen_generic_0000000000000001: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 3430, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wasm_bindgen_13ee6a147c6270a5___JsValue__core_d941ad8722425d2e___result__Result_____wasm_bindgen_13ee6a147c6270a5___JsError___true_);
                return ret;
            },
            __wbindgen_generic_0000000000000002: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("GPUDevice")], shim_idx: 1738, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wasm_bindgen_13ee6a147c6270a5___sys__JsNullable_wgpu_f3f036a789e6748e___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_d941ad8722425d2e___result__Result_____wasm_bindgen_13ee6a147c6270a5___JsError___true_);
                return ret;
            },
            __wbindgen_generic_0000000000000003: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("GPUUncapturedErrorEvent")], shim_idx: 1739, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wgpu_f3f036a789e6748e___backend__webgpu__webgpu_sys__gen_GpuDeviceLostInfo__GpuDeviceLostInfo______true_);
                return ret;
            },
            __wbindgen_generic_0000000000000004: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("any")], shim_idx: 1738, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wasm_bindgen_13ee6a147c6270a5___sys__JsNullable_wgpu_f3f036a789e6748e___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_d941ad8722425d2e___result__Result_____wasm_bindgen_13ee6a147c6270a5___JsError___true__79);
                return ret;
            },
            __wbindgen_generic_0000000000000005: function(arg0, arg1) {
                // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("undefined")], shim_idx: 1738, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
                const ret = makeMutClosure(arg0, arg1, wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wasm_bindgen_13ee6a147c6270a5___sys__JsNullable_wgpu_f3f036a789e6748e___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_d941ad8722425d2e___result__Result_____wasm_bindgen_13ee6a147c6270a5___JsError___true__80);
                return ret;
            },
            __wbindgen_generic_0000000000000006: function(arg0) {
                // Cast intrinsic for `F64 -> Externref`.
                const ret = arg0;
                return ret;
            },
            __wbindgen_generic_0000000000000007: function(arg0, arg1) {
                // Cast intrinsic for `Ref(Slice(F32)) -> NamedExternref("Float32Array")`.
                const ret = getArrayF32FromWasm0(arg0, arg1);
                return ret;
            },
            __wbindgen_generic_0000000000000008: function(arg0, arg1) {
                // Cast intrinsic for `Ref(Slice(I16)) -> NamedExternref("Int16Array")`.
                const ret = getArrayI16FromWasm0(arg0, arg1);
                return ret;
            },
            __wbindgen_generic_0000000000000009: function(arg0, arg1) {
                // Cast intrinsic for `Ref(Slice(I32)) -> NamedExternref("Int32Array")`.
                const ret = getArrayI32FromWasm0(arg0, arg1);
                return ret;
            },
            __wbindgen_generic_000000000000000a: function(arg0, arg1) {
                // Cast intrinsic for `Ref(Slice(I8)) -> NamedExternref("Int8Array")`.
                const ret = getArrayI8FromWasm0(arg0, arg1);
                return ret;
            },
            __wbindgen_generic_000000000000000b: function(arg0, arg1) {
                // Cast intrinsic for `Ref(Slice(U16)) -> NamedExternref("Uint16Array")`.
                const ret = getArrayU16FromWasm0(arg0, arg1);
                return ret;
            },
            __wbindgen_generic_000000000000000c: function(arg0, arg1) {
                // Cast intrinsic for `Ref(Slice(U32)) -> NamedExternref("Uint32Array")`.
                const ret = getArrayU32FromWasm0(arg0, arg1);
                return ret;
            },
            __wbindgen_generic_000000000000000d: function(arg0, arg1) {
                // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
                const ret = getArrayU8FromWasm0(arg0, arg1);
                return ret;
            },
            __wbindgen_generic_000000000000000e: function(arg0, arg1) {
                // Cast intrinsic for `Ref(String) -> Externref`.
                const ret = getStringFromWasm0(arg0, arg1);
                return ret;
            },
            __wbindgen_init_externref_table: function() {
                const table = wasm.__wbindgen_externrefs;
                table.set(0, undefined);
                table.set(1024, undefined);
                table.set(1025, null);
                table.set(1026, true);
                table.set(1027, false);
            },
        };
        return {
            __proto__: null,
            "./website_bg.js": import0,
        };
    }

    function wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wgpu_f3f036a789e6748e___backend__webgpu__webgpu_sys__gen_GpuDeviceLostInfo__GpuDeviceLostInfo______true_(arg0, arg1, arg2) {
        wasm.wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wgpu_f3f036a789e6748e___backend__webgpu__webgpu_sys__gen_GpuDeviceLostInfo__GpuDeviceLostInfo______true_(arg0, arg1, arg2);
    }

    function wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wasm_bindgen_13ee6a147c6270a5___JsValue__core_d941ad8722425d2e___result__Result_____wasm_bindgen_13ee6a147c6270a5___JsError___true_(arg0, arg1, arg2) {
        const ret = wasm.wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wasm_bindgen_13ee6a147c6270a5___JsValue__core_d941ad8722425d2e___result__Result_____wasm_bindgen_13ee6a147c6270a5___JsError___true_(arg0, arg1, arg2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }

    function wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wasm_bindgen_13ee6a147c6270a5___sys__JsNullable_wgpu_f3f036a789e6748e___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_d941ad8722425d2e___result__Result_____wasm_bindgen_13ee6a147c6270a5___JsError___true_(arg0, arg1, arg2) {
        const ret = wasm.wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wasm_bindgen_13ee6a147c6270a5___sys__JsNullable_wgpu_f3f036a789e6748e___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_d941ad8722425d2e___result__Result_____wasm_bindgen_13ee6a147c6270a5___JsError___true_(arg0, arg1, arg2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }

    function wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wasm_bindgen_13ee6a147c6270a5___sys__JsNullable_wgpu_f3f036a789e6748e___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_d941ad8722425d2e___result__Result_____wasm_bindgen_13ee6a147c6270a5___JsError___true__79(arg0, arg1, arg2) {
        const ret = wasm.wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wasm_bindgen_13ee6a147c6270a5___sys__JsNullable_wgpu_f3f036a789e6748e___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_d941ad8722425d2e___result__Result_____wasm_bindgen_13ee6a147c6270a5___JsError___true__79(arg0, arg1, arg2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }

    function wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wasm_bindgen_13ee6a147c6270a5___sys__JsNullable_wgpu_f3f036a789e6748e___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_d941ad8722425d2e___result__Result_____wasm_bindgen_13ee6a147c6270a5___JsError___true__80(arg0, arg1, arg2) {
        const ret = wasm.wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___wasm_bindgen_13ee6a147c6270a5___sys__JsNullable_wgpu_f3f036a789e6748e___backend__webgpu__webgpu_sys__gen_GpuError__GpuError___core_d941ad8722425d2e___result__Result_____wasm_bindgen_13ee6a147c6270a5___JsError___true__80(arg0, arg1, arg2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }

    function wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___js_sys_b1c51206aec32abd___Function_fn_wasm_bindgen_13ee6a147c6270a5___JsValue_____wasm_bindgen_13ee6a147c6270a5___sys__Undefined___js_sys_b1c51206aec32abd___Function_fn_wasm_bindgen_13ee6a147c6270a5___JsValue_____wasm_bindgen_13ee6a147c6270a5___sys__Undefined_______true_(arg0, arg1, arg2, arg3) {
        wasm.wasm_bindgen_13ee6a147c6270a5___convert__closures_____invoke___js_sys_b1c51206aec32abd___Function_fn_wasm_bindgen_13ee6a147c6270a5___JsValue_____wasm_bindgen_13ee6a147c6270a5___sys__Undefined___js_sys_b1c51206aec32abd___Function_fn_wasm_bindgen_13ee6a147c6270a5___JsValue_____wasm_bindgen_13ee6a147c6270a5___sys__Undefined_______true_(arg0, arg1, arg2, arg3);
    }


    const __wbindgen_enum_GpuAddressMode = ["clamp-to-edge", "repeat", "mirror-repeat"];


    const __wbindgen_enum_GpuAutoLayoutMode = ["auto"];


    const __wbindgen_enum_GpuBlendFactor = ["zero", "one", "src", "one-minus-src", "src-alpha", "one-minus-src-alpha", "dst", "one-minus-dst", "dst-alpha", "one-minus-dst-alpha", "src-alpha-saturated", "constant", "one-minus-constant", "src1", "one-minus-src1", "src1-alpha", "one-minus-src1-alpha"];


    const __wbindgen_enum_GpuBlendOperation = ["add", "subtract", "reverse-subtract", "min", "max"];


    const __wbindgen_enum_GpuBufferBindingType = ["uniform", "storage", "read-only-storage"];


    const __wbindgen_enum_GpuCanvasAlphaMode = ["opaque", "premultiplied"];


    const __wbindgen_enum_GpuCanvasToneMappingMode = ["standard", "extended"];


    const __wbindgen_enum_GpuCompareFunction = ["never", "less", "equal", "less-equal", "greater", "not-equal", "greater-equal", "always"];


    const __wbindgen_enum_GpuCullMode = ["none", "front", "back"];


    const __wbindgen_enum_GpuFilterMode = ["nearest", "linear"];


    const __wbindgen_enum_GpuFrontFace = ["ccw", "cw"];


    const __wbindgen_enum_GpuIndexFormat = ["uint16", "uint32"];


    const __wbindgen_enum_GpuLoadOp = ["load", "clear"];


    const __wbindgen_enum_GpuMipmapFilterMode = ["nearest", "linear"];


    const __wbindgen_enum_GpuPowerPreference = ["low-power", "high-performance"];


    const __wbindgen_enum_GpuPrimitiveTopology = ["point-list", "line-list", "line-strip", "triangle-list", "triangle-strip"];


    const __wbindgen_enum_GpuSamplerBindingType = ["filtering", "non-filtering", "comparison"];


    const __wbindgen_enum_GpuStencilOperation = ["keep", "zero", "replace", "invert", "increment-clamp", "decrement-clamp", "increment-wrap", "decrement-wrap"];


    const __wbindgen_enum_GpuStorageTextureAccess = ["write-only", "read-only", "read-write"];


    const __wbindgen_enum_GpuStoreOp = ["store", "discard"];


    const __wbindgen_enum_GpuTextureAspect = ["all", "stencil-only", "depth-only"];


    const __wbindgen_enum_GpuTextureDimension = ["1d", "2d", "3d"];


    const __wbindgen_enum_GpuTextureFormat = ["r8unorm", "r8snorm", "r8uint", "r8sint", "r16unorm", "r16snorm", "r16uint", "r16sint", "r16float", "rg8unorm", "rg8snorm", "rg8uint", "rg8sint", "r32uint", "r32sint", "r32float", "rg16unorm", "rg16snorm", "rg16uint", "rg16sint", "rg16float", "rgba8unorm", "rgba8unorm-srgb", "rgba8snorm", "rgba8uint", "rgba8sint", "bgra8unorm", "bgra8unorm-srgb", "rgb9e5ufloat", "rgb10a2uint", "rgb10a2unorm", "rg11b10ufloat", "rg32uint", "rg32sint", "rg32float", "rgba16unorm", "rgba16snorm", "rgba16uint", "rgba16sint", "rgba16float", "rgba32uint", "rgba32sint", "rgba32float", "stencil8", "depth16unorm", "depth24plus", "depth24plus-stencil8", "depth32float", "depth32float-stencil8", "bc1-rgba-unorm", "bc1-rgba-unorm-srgb", "bc2-rgba-unorm", "bc2-rgba-unorm-srgb", "bc3-rgba-unorm", "bc3-rgba-unorm-srgb", "bc4-r-unorm", "bc4-r-snorm", "bc5-rg-unorm", "bc5-rg-snorm", "bc6h-rgb-ufloat", "bc6h-rgb-float", "bc7-rgba-unorm", "bc7-rgba-unorm-srgb", "etc2-rgb8unorm", "etc2-rgb8unorm-srgb", "etc2-rgb8a1unorm", "etc2-rgb8a1unorm-srgb", "etc2-rgba8unorm", "etc2-rgba8unorm-srgb", "eac-r11unorm", "eac-r11snorm", "eac-rg11unorm", "eac-rg11snorm", "astc-4x4-unorm", "astc-4x4-unorm-srgb", "astc-5x4-unorm", "astc-5x4-unorm-srgb", "astc-5x5-unorm", "astc-5x5-unorm-srgb", "astc-6x5-unorm", "astc-6x5-unorm-srgb", "astc-6x6-unorm", "astc-6x6-unorm-srgb", "astc-8x5-unorm", "astc-8x5-unorm-srgb", "astc-8x6-unorm", "astc-8x6-unorm-srgb", "astc-8x8-unorm", "astc-8x8-unorm-srgb", "astc-10x5-unorm", "astc-10x5-unorm-srgb", "astc-10x6-unorm", "astc-10x6-unorm-srgb", "astc-10x8-unorm", "astc-10x8-unorm-srgb", "astc-10x10-unorm", "astc-10x10-unorm-srgb", "astc-12x10-unorm", "astc-12x10-unorm-srgb", "astc-12x12-unorm", "astc-12x12-unorm-srgb"];


    const __wbindgen_enum_GpuTextureSampleType = ["float", "unfilterable-float", "depth", "sint", "uint"];


    const __wbindgen_enum_GpuTextureViewDimension = ["1d", "2d", "2d-array", "cube", "cube-array", "3d"];


    const __wbindgen_enum_GpuVertexFormat = ["uint8", "uint8x2", "uint8x4", "sint8", "sint8x2", "sint8x4", "unorm8", "unorm8x2", "unorm8x4", "snorm8", "snorm8x2", "snorm8x4", "uint16", "uint16x2", "uint16x4", "sint16", "sint16x2", "sint16x4", "unorm16", "unorm16x2", "unorm16x4", "snorm16", "snorm16x2", "snorm16x4", "float16", "float16x2", "float16x4", "float32", "float32x2", "float32x3", "float32x4", "uint32", "uint32x2", "uint32x3", "uint32x4", "sint32", "sint32x2", "sint32x3", "sint32x4", "unorm10-10-10-2", "unorm8x4-bgra"];


    const __wbindgen_enum_GpuVertexStepMode = ["vertex", "instance"];

    function addToExternrefTable0(obj) {
        const idx = wasm.__externref_table_alloc();
        wasm.__wbindgen_externrefs.set(idx, obj);
        return idx;
    }

    const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
        ? { register: () => {}, unregister: () => {} }
        : new FinalizationRegistry(state => wasm.__wbindgen_destroy_closure(state.a, state.b));

    function debugString(val) {
        switch (typeof val) {
            // primitive types
            case 'number':
            case 'boolean':
            case 'undefined':
            return `${val}`;
            case 'string':
            return JSON.stringify(val);
            case 'bigint':
            return `${val}n`;
            case 'symbol':
            return val.description !== undefined
                ? `Symbol(${val.description})`
                : 'Symbol';
            case 'function':
            return typeof val.name === 'string' && val.name !== ''
                ? `Function(${val.name})`
                : 'Function';
            case 'object': {
                if (val === null) {
                    return 'null';
                }
                // objects
                if (Array.isArray(val)) {
                    return '[' + Array.from(val, debugString).join(', ') + ']';
                }
                // errors
                if (val instanceof Error) {
                    return `${val.name}: ${val.message}\n${val.stack}`;
                }

                const toStringOutput = toString.call(val);
                // Test for built-in
                const className = /\[object ([^\]]+)\]/.exec(toStringOutput)?.[1];

                // Failed to match the standard '[object ClassName]'
                if (className === undefined) {
                    return toStringOutput;
                }
                if (className === 'Object') {
                    // we're a user defined class or Object
                    // JSON.stringify avoids problems with cycles, and is generally much
                    // easier than looping through ownProperties of `val`.
                    try {
                        return 'Object(' + JSON.stringify(val) + ')';
                    } catch (_) {
                        return 'Object';
                    }
                }
                // TODO we could test for more things here, like `Set`s and `Map`s.
                return className;
            }
            // Per MDN: 'This list of values is exhaustive.'
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/typeof#description
        }
    }

    function getArrayF32FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
    }

    function getArrayI16FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getInt16ArrayMemory0().subarray(ptr / 2, ptr / 2 + len);
    }

    function getArrayI32FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getInt32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
    }

    function getArrayI8FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getInt8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
    }

    function getArrayJsValueViewFromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        const mem = getDataViewMemory0();
        const result = [];
        for (let i = ptr; i < ptr + 4 * len; i += 4) {
            result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
        }
        return result;
    }

    function getArrayU16FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getUint16ArrayMemory0().subarray(ptr / 2, ptr / 2 + len);
    }

    function getArrayU32FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
    }

    function getArrayU8FromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
    }

    let cachedDataViewMemory0 = null;
    function getDataViewMemory0() {
        if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
            cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
        }
        return cachedDataViewMemory0;
    }

    let cachedFloat32ArrayMemory0 = null;
    function getFloat32ArrayMemory0() {
        if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
            cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
        }
        return cachedFloat32ArrayMemory0;
    }

    let cachedInt16ArrayMemory0 = null;
    function getInt16ArrayMemory0() {
        if (cachedInt16ArrayMemory0 === null || cachedInt16ArrayMemory0.byteLength === 0) {
            cachedInt16ArrayMemory0 = new Int16Array(wasm.memory.buffer);
        }
        return cachedInt16ArrayMemory0;
    }

    let cachedInt32ArrayMemory0 = null;
    function getInt32ArrayMemory0() {
        if (cachedInt32ArrayMemory0 === null || cachedInt32ArrayMemory0.byteLength === 0) {
            cachedInt32ArrayMemory0 = new Int32Array(wasm.memory.buffer);
        }
        return cachedInt32ArrayMemory0;
    }

    let cachedInt8ArrayMemory0 = null;
    function getInt8ArrayMemory0() {
        if (cachedInt8ArrayMemory0 === null || cachedInt8ArrayMemory0.byteLength === 0) {
            cachedInt8ArrayMemory0 = new Int8Array(wasm.memory.buffer);
        }
        return cachedInt8ArrayMemory0;
    }

    function getStringFromWasm0(ptr, len) {
        ptr = ptr >>> 0;
        return decodeText(getUint8ArrayMemory0().subarray(ptr, ptr + len));
    }

    let cachedUint16ArrayMemory0 = null;
    function getUint16ArrayMemory0() {
        if (cachedUint16ArrayMemory0 === null || cachedUint16ArrayMemory0.byteLength === 0) {
            cachedUint16ArrayMemory0 = new Uint16Array(wasm.memory.buffer);
        }
        return cachedUint16ArrayMemory0;
    }

    let cachedUint32ArrayMemory0 = null;
    function getUint32ArrayMemory0() {
        if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
            cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
        }
        return cachedUint32ArrayMemory0;
    }

    let cachedUint8ArrayMemory0 = null;
    function getUint8ArrayMemory0() {
        if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
            cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
        }
        return cachedUint8ArrayMemory0;
    }

    function handleError(f, args) {
        try {
            return f.apply(this, args);
        } catch (e) {
            const idx = addToExternrefTable0(e);
            wasm.__wbindgen_exn_store(idx);
        }
    }

    function isLikeNone(x) {
        return x === undefined || x === null;
    }

    function makeMutClosure(arg0, arg1, f) {
        const state = { a: arg0, b: arg1, cnt: 1 };
        const real = (...args) => {

            // First up with a closure we increment the internal reference
            // count. This ensures that the Rust closure environment won't
            // be deallocated while we're invoking it.
            state.cnt++;
            const a = state.a;
            state.a = 0;
            try {
                return f(a, state.b, ...args);
            } finally {
                state.a = a;
                real._wbg_cb_unref();
            }
        };
        real._wbg_cb_unref = () => {
            if (--state.cnt === 0) {
                wasm.__wbindgen_destroy_closure(state.a, state.b);
                state.a = 0;
                CLOSURE_DTORS.unregister(state);
            }
        };
        CLOSURE_DTORS.register(real, state, state);
        return real;
    }

    function passArray8ToWasm0(arg, malloc) {
        const ptr = malloc(arg.length * 1, 1) >>> 0;
        getUint8ArrayMemory0().set(arg, ptr / 1);
        WASM_VECTOR_LEN = arg.length;
        return ptr;
    }

    function passStringToWasm0(arg, malloc, realloc) {
        if (realloc === undefined) {
            const buf = cachedTextEncoder.encode(arg);
            const ptr = malloc(buf.length, 1) >>> 0;
            getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
            WASM_VECTOR_LEN = buf.length;
            return ptr;
        }

        let len = arg.length;
        let ptr = malloc(len, 1) >>> 0;

        const mem = getUint8ArrayMemory0();

        let offset = 0;

        for (; offset < len; offset++) {
            const code = arg.charCodeAt(offset);
            if (code > 0x7F) break;
            mem[ptr + offset] = code;
        }
        if (offset !== len) {
            if (offset !== 0) {
                arg = arg.slice(offset);
            }
            ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
            const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
            const ret = cachedTextEncoder.encodeInto(arg, view);

            offset += ret.written;
            ptr = realloc(ptr, len, offset, 1) >>> 0;
        }

        WASM_VECTOR_LEN = offset;
        return ptr;
    }

    function takeFromExternrefTable0(idx) {
        const value = wasm.__wbindgen_externrefs.get(idx);
        wasm.__externref_table_dealloc(idx);
        return value;
    }

    let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true });
    cachedTextDecoder.decode();
    function decodeText(bytes) {
        return cachedTextDecoder.decode(bytes);
    }

    const cachedTextEncoder = new TextEncoder();

    if (!('encodeInto' in cachedTextEncoder)) {
        cachedTextEncoder.encodeInto = function (arg, view) {
            const buf = cachedTextEncoder.encode(arg);
            view.set(buf);
            return {
                read: arg.length,
                written: buf.length
            };
        };
    }

    let WASM_VECTOR_LEN = 0;

    let wasmModule, wasmInstance, wasm;
    function __wbg_finalize_init(instance, module) {
        wasmInstance = instance;
        wasm = instance.exports;
        wasmModule = module;
        cachedDataViewMemory0 = null;
        cachedFloat32ArrayMemory0 = null;
        cachedInt16ArrayMemory0 = null;
        cachedInt32ArrayMemory0 = null;
        cachedInt8ArrayMemory0 = null;
        cachedUint16ArrayMemory0 = null;
        cachedUint32ArrayMemory0 = null;
        cachedUint8ArrayMemory0 = null;
        wasm.__wbindgen_start();
        return wasm;
    }

    async function __wbg_load(module, imports) {
        if (typeof Response === 'function' && module instanceof Response) {
            if (!module.ok) {
                throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
            }

            if (typeof WebAssembly.instantiateStreaming === 'function') {
                try {
                    return await WebAssembly.instantiateStreaming(module, imports);
                } catch (e) {
                    const validResponse = expectedResponseType(module.type);

                    if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                        console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                    } else { throw e; }
                }
            }

            const bytes = await module.arrayBuffer();
            return await WebAssembly.instantiate(bytes, imports);
        } else {
            const instance = await WebAssembly.instantiate(module, imports);

            if (instance instanceof WebAssembly.Instance) {
                return { instance, module };
            } else {
                return instance;
            }
        }

        function expectedResponseType(type) {
            switch (type) {
                case 'basic': case 'cors': case 'default': return true;
            }
            return false;
        }
    }

    function initSync(module) {
        if (wasm !== undefined) return wasm;


        if (module !== undefined) {
            if (Object.getPrototypeOf(module) === Object.prototype) {
                ({module} = module)
            } else {
                console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
            }
        }

        const imports = __wbg_get_imports();
        if (!(module instanceof WebAssembly.Module)) {
            module = new WebAssembly.Module(module);
        }
        const instance = new WebAssembly.Instance(module, imports);
        return __wbg_finalize_init(instance, module);
    }

    let __wbg_init_promise;
    async function __wbg_init(module_or_path) {
        if (wasm !== undefined) return wasm;
        if (__wbg_init_promise === undefined) {
            __wbg_init_promise = __wbg_init_once(module_or_path);
            __wbg_init_promise.catch(() => { __wbg_init_promise = undefined; });
        }
        return __wbg_init_promise;
    }

    async function __wbg_init_once(module_or_path) {

        if (module_or_path !== undefined) {
            if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
                ({module_or_path} = module_or_path)
            } else {
                console.warn('using deprecated parameters for the initialization function; pass a single object instead')
            }
        }

        if (module_or_path === undefined && script_src !== undefined) {
            module_or_path = script_src.replace(/\.js$/, "_bg.wasm");
        }
        const imports = __wbg_get_imports();

        if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
            module_or_path = fetch(module_or_path);
        }

        const { instance, module } = await __wbg_load(await module_or_path, imports);

        return __wbg_finalize_init(instance, module);
    }

    return Object.assign(__wbg_init, { initSync }, exports);
})({ __proto__: null });

// The glue between the browser and the framework (after wasm-bindgen's own, which the build puts in
// front of this file). `mount(stage, opts)` starts the framework in a stage element (the canvas, the
// text agent and the accessibility mirror) and returns what a host can change. The site's pages
// mount once in #stage (`site()` at the end); `<amenui-view>` (embed.js) mounts in its shadow root.
// One view per document: the module keeps one interface.
//
// opts: `load` (a function giving { module, fonts: [sans, pixel, mono], texts, glue } as promises;
// `texts`, [name, body] pairs, may be left out; `glue` is the module's wasm-bindgen glue, the global
// `wasm_bindgen` when left out), `faces` (the faces the module does not carry, name → URL: each
// is fetched once, when a page first uses it), `texts` ({ texts: base URL, textsV: version }: where the
// texts the module does not carry are, each fetched when the module first asks for it), `route` (the
// first page: a path such as `components/button/`, or a hash name such as `#button`), `keys` (where
// key events come from; the window by default), `onRoute(path, hash)` (the page changed inside it),
// `started(module)` (after the module started, before its first frame), `onFrame()` (the first frame
// is drawn), `frame()` (after every frame the loop runs), `after()` (after every handled input).
//
// The pictures (photographs drawn in marks) are made when the page first draws one (`pictures`).
async function mount(stage, opts) {
  const root = stage.getRootNode();
  const active = () => (root.activeElement !== undefined ? root.activeElement : document.activeElement);
  const canvas = stage.querySelector('.c');
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const fit = () => { const r = stage.getBoundingClientRect(); canvas.width = Math.max(1, Math.floor(r.width * dpr)); canvas.height = Math.max(1, Math.floor(r.height * dpr)); };
  fit();
  const got = opts.load();
  const t0 = performance.now();
  const w = got.glue ? await got.glue : wasm_bindgen;
  // Tools that read the module (frame-time probes) find it by its usual name.
  if (got.glue) self.wasm_bindgen = w;
  await w({ module_or_path: got.module });
  const compiled = performance.now() - t0;
  const [sans, pixel, mono] = (await got.fonts).map(b => new Uint8Array(b));
  // The texts the module does not carry: those given go in before it starts; one it asks for later is
  // fetched once and the page is drawn again when it comes. What the first page asks for comes before
  // its first frame.
  const textsAsked = new Set(), redraw = { kick: () => {} };
  const putTexts = list => { for (const [n, b] of list || []) { textsAsked.add(n); if (typeof b === 'string') w.put_text(n, b); } };
  const fetchTexts = () => {
    const want = w.wanted_texts ? w.wanted_texts().split('\n').filter(n => n && !textsAsked.has(n)) : [];
    if (!want.length || !opts.texts) return null;
    return Promise.all(want.map(n => { textsAsked.add(n); return fetch(opts.texts.texts + n + '?' + opts.texts.textsV).then(r => r.ok ? r.text() : null).then(b => { if (b !== null) { w.put_text(n, b); redraw.kick(); } }, () => {}); }));
  };
  putTexts(await got.texts);
  await w.start(canvas, sans, pixel, mono, dpr, opts.route || '/');
  const asking = fetchTexts();
  if (asking) await asking;
  // After start: the pictures the first page draws go to the renderer, which start makes.
  w.pictures();
  // The page's scheme (AA.10, web/scheme.js sets it on the root before the first paint), before the
  // first frame; on a page without one (an embedding) the canvas keeps its own.
  const theme = document.documentElement.dataset.theme ? () => document.documentElement.dataset.theme === 'light' : null;
  if (theme && w.set_scheme) w.set_scheme(theme(), NaN, NaN);
  if (opts.started) opts.started(w);
  canvas.focus({ preventScroll: true });
  const now = () => performance.now() / 1000;
  const at = e => { const r = canvas.getBoundingClientRect(); return [e.clientX - r.left, e.clientY - r.top]; };
  let shown = null;
  const sync = () => { const p = w.path(); if (p === shown) return; shown = p; if (w.pace) w.pace(); if (opts.onRoute) opts.onRoute(p, w.hash()); };
  sync();
  // Frames on demand: the animation-frame loop runs only while the framework needs frames; when it
  // does not, the loop stops, and input or a timed wake (the caret's blink, a clock) starts it again.
  let raf = 0, timer = 0, first = true;
  const loop = () => {
    raf = 0; if (timer) { clearTimeout(timer); timer = 0; }
    const t = now();
    let d = w.frame(t);
    // A picture drawn for the first time is made, and the frame drawn again before it is shown.
    if (w.pictures()) d = w.frame(t);
    fetchFaces();
    fetchTexts();
    axUpdate();
    if (first && w.frames()[0] > 0) { first = false; if (opts.onFrame) opts.onFrame(); }
    if (opts.frame) opts.frame();
    // A wait ends on an animation frame: the timer wakes a refresh early (as amenui.js does), so a frame
    // due in one refresh (30 a second on a 60 Hz display) is drawn on that refresh and not the one after.
    if (d === 0) raf = requestAnimationFrame(loop);
    else if (d > 0) timer = setTimeout(kick, Math.max(0, d * 1000 - 20));
  };
  const kick = () => { if (!raf) raf = requestAnimationFrame(loop); };
  redraw.kick = kick;
  // The renderer's warm-up goes on in idle time, whether or not the page draws (a page at rest draws
  // nothing): each idle period makes the next pieces and draws nothing; a frame only when the look's
  // material can be drawn (2). It stops once nothing is left (no 1), never on a call that traps.
  // Without requestIdleCallback, an animation frame that does warm-up work only.
  const idle = window.requestIdleCallback ? f => requestIdleCallback(f, { timeout: 200 }) : f => requestAnimationFrame(() => f(null));
  const warmIdle = d => {
    if (!w.warm_idle) return;
    const ms = d && !d.didTimeout ? Math.max(1, Math.min(d.timeRemaining(), 6)) : 6;
    // A call that traps leaves work behind: the next idle period tries again. A module busy in another
    // call answers 1 (more to do), so only a call that has nothing left ends the loop.
    let r = 1;
    try { r = w.warm_idle(ms, now()); } catch (e) { r = 1; }
    if (r & 2) kick();
    if (r & 1) idle(warmIdle);
  };
  idle(warmIdle);
  // Faces the page uses that have not come: each fetched once; when its bytes come the page takes it
  // (each line sweeps over to it).
  const asked = new Set();
  const fetchFaces = () => {
    if (!opts.faces || !w.faces_wanted) return;
    for (const name of JSON.parse(w.faces_wanted())) {
      if (asked.has(name) || !opts.faces[name]) continue;
      asked.add(name);
      fetch(opts.faces[name]).then(r => { if (!r.ok) throw new Error(r.status); return r.arrayBuffer(); })
        .then(b => { w.add_face(name, new Uint8Array(b), now()); kick(); }, () => {});
    }
  };
  // Text: while a text field has focus the hidden text element sits at its caret and has the keyboard.
  const ta = stage.querySelector('.ta');
  const syncText = () => {
    const r = w.ime_rect();
    if (r.length) {
      ta.style.left = r[0] + 'px'; ta.style.top = r[1] + 'px'; ta.style.height = Math.max(r[3], 10) + 'px'; ta.style.fontSize = Math.max(r[3] * 0.8, 10) + 'px';
      if (active() !== ta) ta.focus({ preventScroll: true });
    } else if (active() === ta) canvas.focus({ preventScroll: true });
    canvas.style.cursor = w.cursor();
  };
  // The accessibility mirror, kept in step with what the framework reports changed.
  const ax = stage.querySelector('.a11y'), els = new Map();
  const axUpdate = () => {
    const c = canvas.getBoundingClientRect();
    for (const op of JSON.parse(w.a11y_update())) {
      if (op.remove !== undefined) { const e = els.get(op.remove); if (e) { e.remove(); els.delete(op.remove); } continue; }
      if (op.focus !== undefined) {
        // Focus follows only while assistive technology is in the mirror; otherwise the canvas (or the
        // text element) keeps the keyboard.
        const e = op.focus && els.get(op.focus);
        if (e && ax.contains(active()) && active() !== e) e.focus({ preventScroll: true });
        continue;
      }
      const n = op.set;
      let e = els.get(n.key);
      if (!e) {
        e = document.createElement('div');
        els.set(n.key, e);
        e.addEventListener('click', ev => { ev.stopPropagation(); w.a11y_activate(n.key, now()); after(); });
        e.addEventListener('focus', () => { w.a11y_focus(n.key, now()); after(); });
      }
      const parent = (n.parent && els.get(n.parent)) || ax;
      if (e.parentNode !== parent) parent.appendChild(e);
      for (const a of ['role', 'aria-label', 'aria-checked', 'aria-selected', 'aria-valuenow', 'aria-valuemin', 'aria-valuemax', 'aria-modal', 'aria-level', 'aria-disabled', 'aria-expanded', 'aria-description', 'aria-live', 'aria-invalid', 'aria-required', 'tabindex']) e.removeAttribute(a);
      if (n.role === 'text') { if (e.textContent !== n.name) e.textContent = n.name; }
      else {
        e.setAttribute('role', n.role);
        e.setAttribute('aria-label', n.name);
        if (n.role === 'textbox') e.textContent = n.text || '';
      }
      if (n.checked !== undefined) e.setAttribute('aria-checked', n.checked);
      if (n.selected !== undefined) e.setAttribute('aria-selected', n.selected);
      if (n.value) { e.setAttribute('aria-valuenow', n.value[0]); e.setAttribute('aria-valuemin', n.value[1]); e.setAttribute('aria-valuemax', n.value[2]); }
      if (n.modal) e.setAttribute('aria-modal', 'true');
      if (n.level) e.setAttribute('aria-level', n.level);
      if (n.disabled) e.setAttribute('aria-disabled', 'true');
      if (n.expanded !== undefined) e.setAttribute('aria-expanded', n.expanded);
      if (n.description) e.setAttribute('aria-description', n.description);
      if (n.live) e.setAttribute('aria-live', n.live);
      if (n.invalid) e.setAttribute('aria-invalid', 'true');
      if (n.required) e.setAttribute('aria-required', 'true');
      if (n.focusable) e.setAttribute('tabindex', '-1');
      const r = n.rect;
      e.style.left = (c.left + r[0]) + 'px'; e.style.top = (c.top + r[1]) + 'px'; e.style.width = Math.max(r[2], 1) + 'px'; e.style.height = Math.max(r[3], 1) + 'px';
    }
  };
  // A copy button in the page asks for the clipboard: hand its text to the browser after the event.
  const clip = () => { const t = w.take_clipboard(); if (t != null && navigator.clipboard) navigator.clipboard.writeText(t).catch(() => {}); };
  // The readout's open / compact choice outlives the page (per browser; nothing breaks without storage).
  const READOUT = 'amenui.readout';
  let readout = 0;
  try { const v = localStorage.getItem(READOUT); if (v !== null) { w.set_readout(+v === 2 ? 2 : 0); } } catch (e) {}
  readout = w.readout();
  const keepReadout = () => { const v = w.readout(); if (v !== readout) { readout = v; try { localStorage.setItem(READOUT, String(v)); } catch (e) {} } };
  const after = () => { sync(); syncText(); axUpdate(); clip(); keepReadout(); if (opts.after) opts.after(); kick(); };
  // A change of scheme: from the switch it spreads from where it was pressed, from the system it is a cut.
  if (theme && w.set_scheme) document.addEventListener('amenui:scheme', e => {
    const c = canvas.getBoundingClientRect(), f = e.detail && e.detail.from;
    w.set_scheme(theme(), f ? f[0] - c.left : NaN, f ? f[1] - c.top : NaN);
    after();
  });
  // Mouse, pen and touch come as pointer events (kind 0, 2, 1); the pointer is captured while it is
  // down, so a drag keeps going outside the canvas.
  const kind = e => e.pointerType === 'touch' ? 1 : e.pointerType === 'pen' ? 2 : 0;
  canvas.addEventListener('pointermove', e => {
    for (const c of (e.getCoalescedEvents ? e.getCoalescedEvents() : [e])) w.pointer_move(e.pointerId, kind(e), ...at(c), now(), (c.timeStamp || e.timeStamp) / 1000);
    canvas.style.cursor = w.cursor(); kick();
  });
  canvas.addEventListener('pointerleave', e => { if (kind(e) !== 1) { w.pointer_leave(now()); kick(); } });
  // The page decides where focus goes (the canvas, or the text element when a field is pressed), so the
  // browser's own focus change on press is suppressed. Only the primary button presses.
  canvas.addEventListener('pointerdown', e => {
    e.preventDefault();
    if (kind(e) === 0 && e.button !== 0) return;
    canvas.focus({ preventScroll: true }); canvas.setPointerCapture(e.pointerId);
    const [x, y] = at(e);
    if (kind(e) !== 1) w.pointer_move(e.pointerId, kind(e), x, y, now(), e.timeStamp / 1000);
    w.pointer_down(e.pointerId, kind(e), x, y, e.shiftKey, now(), e.timeStamp / 1000); after();
  });
  canvas.addEventListener('pointerup', e => { if (kind(e) === 0 && e.button !== 0) return; w.pointer_up(e.pointerId, kind(e), ...at(e), now()); after(); });
  canvas.addEventListener('pointercancel', e => { w.pointer_cancel(e.pointerId, now()); after(); });
  canvas.addEventListener('contextmenu', e => e.preventDefault());
  // Wheel deltas in pixels; lines are 40 px (as on the native window), pages the canvas's height. A
  // pinch arrives as a wheel with Ctrl: it is not a scroll, and the page is not zoomed.
  canvas.addEventListener('wheel', e => {
    e.preventDefault();
    if (e.ctrlKey) return;
    const k = e.deltaMode === 1 ? 40 : e.deltaMode === 2 ? canvas.clientHeight : 1;
    w.wheel(...at(e), e.deltaX * k, e.deltaY * k, now()); kick();
  }, { passive: false });
  // Keys go to the canvas only while focus is on it (or on nothing): the page's own links and buttons
  // keep Tab, Enter and the rest.
  const ours = () => { const a = active(); return !a || a === canvas || a === ta || a === document.body || ax.contains(a); };
  (opts.keys || window).addEventListener('keydown', e => {
    // Keys the input method is composing with are its own; keys the page has taken (the search's `/`
    // and ⌘K / Ctrl+K) are not the canvas's.
    if (e.isComposing || e.keyCode === 229 || e.defaultPrevented) return;
    const cmd = e.ctrlKey || e.metaKey;
    if (e.key === 'F2' && !ours()) { e.preventDefault(); w.set_readout(w.readout() === 2 ? 0 : 2); after(); return; }
    if (!ours()) return;
    if (cmd) {
      const k = e.key.toLowerCase();
      // Copy, cut and paste arrive as clipboard events; other shortcuts belong to the browser.
      if (!['a', 'z', 'y'].includes(k)) return;
      if (w.key_mods(e.key, e.shiftKey, true, e.altKey, now())) e.preventDefault();
      after(); return;
    }
    const text = w.text_focused();
    if (e.altKey && !text) return;
    // In a text field characters arrive as input events (so the input method can take them).
    if (text && e.key.length === 1) return;
    if (w.key_mods(e.key, e.shiftKey, false, e.altKey, now()) || e.key === 'Tab') e.preventDefault();
    after();
  });
  ta.addEventListener('input', e => {
    if (e.isComposing) return;
    if (e.inputType === 'insertLineBreak') w.key_mods('Enter', false, false, false, now());
    // A composition's text comes through the composition events.
    else if (e.data && e.inputType !== 'insertCompositionText') w.text_input(e.data, now());
    ta.value = ''; after();
  });
  ta.addEventListener('compositionupdate', e => { w.ime_preedit(e.data || '', now()); after(); });
  ta.addEventListener('compositionend', e => { w.ime_commit(e.data || '', now()); ta.value = ''; after(); });
  document.addEventListener('copy', e => { if (!ours()) return; const t = w.copy(); if (t != null) { e.clipboardData.setData('text/plain', t); e.preventDefault(); } });
  document.addEventListener('cut', e => { if (!ours()) return; const t = w.cut(now()); if (t != null) { e.clipboardData.setData('text/plain', t); e.preventDefault(); after(); } });
  document.addEventListener('paste', e => { if (!ours()) return; const t = e.clipboardData.getData('text/plain'); if (t && w.paste(t, now())) { e.preventDefault(); after(); } });
  (opts.keys || window).addEventListener('keyup', e => { if (ours()) { w.key_up(e.key, now()); after(); } });
  // Reduced motion: the Lab's living pieces and the home page's idle turn rest.
  const rm = window.matchMedia ? window.matchMedia('(prefers-reduced-motion: reduce)') : null;
  if (rm) { w.set_reduced_motion(rm.matches); if (rm.addEventListener) rm.addEventListener('change', e => { w.set_reduced_motion(e.matches); kick(); }); }
  // No hover (touch): the Lab index runs the cell nearest the centre.
  const hv = window.matchMedia && w.set_hover ? window.matchMedia('(hover: hover)') : null;
  if (hv) { w.set_hover(hv.matches); if (hv.addEventListener) hv.addEventListener('change', e => { w.set_hover(e.matches); kick(); }); }
  // A new size clears the canvas, and the observer runs after this refresh's animation frame and before
  // its paint: the frame is drawn again here, at the new size, so no refresh shows an empty canvas.
  new ResizeObserver(() => {
    fit(); w.resize(canvas.width, canvas.height, dpr); if (opts.onResize) opts.onResize();
    if (raf) { cancelAnimationFrame(raf); raf = 0; }
    loop();
  }).observe(stage);
  kick();
  // What a host can change from outside: the page (a path, a hash name or a look:v3: recipe), the
  // theme and material by name, the language by tag.
  return {
    w, compiled, kick,
    route: h => { const ok = w.route(h); if (ok) w.reset_stage(); w.pictures(); after(); return ok; },
    look: (theme, material) => { w.set_look(theme || '', material || ''); after(); },
    locale: tag => { w.set_locale(tag || ''); after(); },
    focus: () => canvas.focus({ preventScroll: true }),
  };
}

// The site's own pages: the shell (web/shell.html) has started the downloads and set `AMENUI`.
function site() {
  const A = window.AMENUI, D = document, root = D.documentElement;
  const stage = D.getElementById('stage');
  if (!A || !stage) return;
  window.__labMounted = true;
  const full = p => A.base + A.prefix + p;
  let api = null, intro = false;
  const isHome = () => root.classList.contains('home');
  // The home page's stage, laid out by the page: the canvas draws in the same rectangle.
  const place = () => {
    if (!api) return;
    if (!isHome()) { api.w.set_stage(0, 0, 0, 0, -1); return; }
    const c = stage.getBoundingClientRect(), [x0, y0, x1, y1] = A.stage(), cy = A.centre();
    api.w.set_stage(x0 - c.left, y0 - c.top, x1 - c.left, y1 - c.top, cy < 0 ? -1 : cy - c.top);
    api.kick();
  };
  // The rail row: ▸, the four states, the body's form and seed, and the door to the current state's
  // section (`skip ›` while the walk plays, `continue ›` once a person has taken it over).
  const rail = D.querySelector('.rail'), door = rail.querySelector('.door');
  const DOORS = { 1: ['docs/', 'docs →'], 2: ['components/', 'components →'], 3: ['lab/', 'lab →'], 4: ['works/', 'works →'] };
  const NAMES = { 1: 'source', 2: 'interface', 3: 'material', 4: 'body' };
  const INTRO = 'amenui.intro', VERSION = '1';
  const walk = () => api ? api.w.hero_walk() : [0, 0, 1];
  // The rail and the address read the scene's state after every frame, so they name what the stage
  // draws however the walk ends (and whenever a late press lands).
  let railWas = '';
  const followRail = () => { if (!api || !isHome()) return; const k = walk().join() + (intro ? 'i' : ''); if (k === railWas) return; railWas = k; showRail(); keepHash(); };
  const showRail = () => {
    if (!api || !isHome()) return;
    const [playing, seen, at] = walk();
    if (seen) { try { localStorage.setItem(INTRO, VERSION); } catch (e) {} }
    const st = intro ? 1 : at;
    for (const b of rail.querySelectorAll('.st')) b.setAttribute('aria-pressed', String(b.dataset.state === NAMES[st]));
    rail.classList.toggle('body', st === 4 && !intro && !playing);
    rail.querySelector('.seed').textContent = String(api.w.hero_seed()).padStart(2, '0');
    delete door.dataset.skip; delete door.dataset.play;
    if (intro || playing === 1) { door.textContent = 'skip ›'; door.dataset.skip = '1'; door.removeAttribute('href'); }
    else if (playing === 2) { door.textContent = 'continue ›'; door.dataset.play = '1'; door.removeAttribute('href'); }
    else { const [r, t] = DOORS[st]; door.textContent = t; door.dataset.route = r; door.href = full(r); }
  };
  // The state is in the address (`#source`, `#interface`, `#material`, `#body/cloud/08`; none for
  // the defaults), replaced, not added to the history.
  const keepHash = () => {
    if (!api || !isHome() || intro || walk()[0] === 1) return;
    const st = api.w.hero(), want = st ? '#' + st : '';
    if (location.hash !== want && !location.hash.startsWith('#look:')) history.replaceState(history.state, '', location.pathname + want);
  };
  const press = what => { api.w.hero_press(what, performance.now() / 1000); showRail(); keepHash(); api.kick(); };
  const skip = () => { if (intro) { intro = false; api.w.set_hero('', 0); } else press('skip'); try { localStorage.setItem(INTRO, VERSION); } catch (e) {} showRail(); api.kick(); };
  rail.addEventListener('click', e => {
    const b = e.target.closest('button, a');
    if (!b || !api) return;
    if (b.dataset.skip) { e.preventDefault(); skip(); }
    else if (b.dataset.play || b.classList.contains('play')) { e.preventDefault(); press('play'); }
    else if (b.dataset.state) press(b.dataset.state);
    else if (b.dataset.seed) press(+b.dataset.seed > 0 ? 'seed+' : 'seed-');
  });
  // Esc, or a scroll of more than 8 px, skips the walk.
  D.addEventListener('keydown', e => { if (e.key === 'Escape' && api && isHome() && (intro || walk()[0] === 1)) skip(); });
  addEventListener('scroll', () => { if (api && isHome() && scrollY > 8 && (intro || walk()[0] === 1)) skip(); }, { passive: true });
  const routed = (path, hash) => {
    // The docs are HTML pages: a docs address reached inside the app is a navigation to its page.
    if (path.startsWith('docs/')) { location.assign(full(path)); return; }
    A.mark(path);
    if (location.pathname !== full(path)) history.pushState({ path }, '', full(path) + (location.hash.startsWith('#look:') ? location.hash : ''));
    const t = api ? api.w.title() : '';
    D.title = path === '' ? 'amenui' : (t ? t + ' · amenui' : 'amenui');
    place(); showRail();
  };
  // Nothing turns by itself while the tab is hidden or less than half the stage is in view (AA.14).
  let hidden = false, inView = true;
  const away = () => { if (!api || !api.w.hero_press) return; api.w.hero_press(hidden || !inView ? 'hidden' : 'shown', performance.now() / 1000); api.kick(); };
  D.addEventListener('visibilitychange', () => { hidden = D.hidden; away(); });
  const hs = D.getElementById('hstage');
  if (hs && window.IntersectionObserver) new IntersectionObserver(es => { for (const e of es) inView = e.intersectionRatio >= 0.5; away(); }, { threshold: [0, 0.5, 1] }).observe(hs);
  // On a phone the page scrolls under a vertical drag, except one that starts on the body: that turns it.
  D.getElementById('main').addEventListener('touchstart', e => {
    if (!api || !isHome() || e.touches.length !== 1) return;
    const c = api.w.hero_circle(), r = e.target.getBoundingClientRect(), t = e.touches[0];
    if (c.length === 3 && Math.hypot(t.clientX - r.left - c[0], t.clientY - r.top - c[1]) < c[2]) e.preventDefault();
  }, { passive: false });
  const status = D.querySelector('.status'), f2 = status.querySelector('.f2'), quality = status.querySelector('.quality');
  let running = false, lastDrawn = -1, lastLine = '';
  // `F2` opens the full readout (the canvas draws it over the stage's bottom right); while it is open
  // the quality choice is beside the line.
  const keepReadout = () => {
    if (!api) return;
    const open = api.w.readout() === 2, q = api.w.quality();
    f2.setAttribute('aria-expanded', String(open));
    quality.hidden = !open;
    status.classList.toggle('open', open);
    for (const b of quality.querySelectorAll('button')) b.setAttribute('aria-pressed', String(+b.dataset.q === q));
  };
  f2.addEventListener('click', () => { if (!api) return; api.w.set_readout(api.w.readout() === 2 ? 0 : 2); keepReadout(); api.kick(); });
  quality.addEventListener('click', e => { const b = e.target.closest('button'); if (!b || !api) return; api.w.set_quality(+b.dataset.q); keepReadout(); api.kick(); });
  // A piece of Works (M.2): the bar and the status line take the piece's tone, and fade with the
  // piece's chrome after 3 s without input. Reached from another page, the tone changes at 210 ms (M.5).
  const bar = D.querySelector('.bar');
  const tones = [bar.dataset.tone || '', status.dataset.tone || ''];
  let toneTimer = 0, resting = false;
  const keepWork = () => {
    if (!api || !api.w.works_state) return;
    const [t, idle, rest, f] = api.w.works_state();
    root.classList.toggle('idle', !!idle);
    if (!!rest !== resting) { resting = !!rest; keepStatus(); }
    const name = v => v === 1 ? 'light' : v === 2 ? 'dark' : '';
    const want = [name(t), name(f)];
    if (want[0] === tones[0] && want[1] === tones[1]) return;
    tones[0] = want[0]; tones[1] = want[1]; clearTimeout(toneTimer);
    toneTimer = setTimeout(() => { [bar, status].forEach((e, k) => { if (tones[k]) e.dataset.tone = tones[k]; else delete e.dataset.tone; }); }, want[0] ? 210 : 0);
  };
  const keepStatus = () => {
    if (!api || !running) return;
    keepReadout();
    if (resting) { A.line(A.t.rest); return; }
    // The rate is there only while frames are being drawn; a field with nothing to report is left out.
    // The one-line readout (AA.6): `idle` at rest, the rate and a frame's cost while frames are drawn.
    const drawn = api.w.frames()[0], c = api.w.frame_costs();
    const fps = drawn !== lastDrawn && c[3] > 0 ? Math.round(1000 / c[3]) : 0;
    lastDrawn = drawn;
    const stage = api.w.stage();
    const line = `<b>${api.w.backend()}</b> · ` + (fps ? `<b>${fps}</b> ${A.t.fps} · <b>${c[1].toFixed(1)}</b> ms · ` : 'idle · ') + (stage ? `<b>${stage}</b> · ` : '') + `<b>${api.w.glyphs().toLocaleString('en')}</b> ${A.t.glyphs}`;
    if (line !== lastLine) { lastLine = line; A.line(line); }
  };
  const hash = location.hash.startsWith('#look:') ? '' : decodeURIComponent(location.hash.slice(1));
  // The module the shell chose (`A.kind`): the WebGPU-only one where the browser has WebGPU, else the
  // full one, which also draws with WebGL2. If the WebGPU-only one cannot start (no adapter, no device,
  // or anything else), the full one is tried on a fresh canvas before any failure is shown.
  const start = () => mount(stage, {
    // The Lab's index opened at a step (`/lab/#ink`) takes the step with it.
    route: A.route === '' ? '/' : A.route === 'lab/' && hash ? 'lab/#' + hash : A.route,
    load: () => ({ module: A.wasm, glue: A.glue, fonts: A.fonts, texts: A.texts }), faces: A.files.faces, texts: A.files,
    // Before the first frame: the stage, and the state from the address (none: the short path).
    started: w => {
      api = { w, kick: () => {} };
      // How the last frame was drawn, for a log from the console: `AMENUI.framePath()`.
      A.framePath = () => (api && api.w.frame_path ? api.w.frame_path() : '');
      // The material pass made for the page's material forced on (true), off (false) or left to the
      // renderer (null), for an A/B from the console: `AMENUI.specForce(true)`.
      A.specForce = (v) => { if (api && api.w.spec_force) { api.w.spec_force(v === true ? true : v === false ? false : undefined); api.kick(); } };
      if (w.set_status_line) w.set_status_line(true);
      A.line(A.t.starting.replace('{gpu}', navigator.gpu ? 'webgpu' : 'webgl2'));
      // No state in the address: the full walk on a first visit, the short path after (and with reduced
      // motion, which plays no walk).
      if (A.route === '') {
        let seen = false;
        try { seen = localStorage.getItem(INTRO) === VERSION; } catch (e) {}
        const whole = hash === '' && !seen && !matchMedia('(prefers-reduced-motion: reduce)').matches;
        intro = hash === '' && !whole;
        w.set_hero(hash, hash === '' ? (whole ? 2 : 1) : 0);
      }
      place();
    },
    onRoute: (p, h) => routed(p, h),
    onResize: () => { place(); A.draw(); },
    after: () => {
      showRail(); keepHash(); keepReadout(); keepWork();
      // A press that asked for an HTML page (a piece's recipe) goes there.
      const to = api && api.w.take_navigate ? api.w.take_navigate() : '';
      if (to) location.assign(full(to));
    },
    frame: () => { followRail(); keepWork(); },
    onFrame: () => {
      A.done(api.w.backend(), api.w.glyphs());
      if (intro) setTimeout(() => { intro = false; showRail(); keepHash(); }, 1400);
      showRail();
      // The running line is read once, not every second.
      setTimeout(() => { running = true; f2.hidden = false; status.querySelector('.line').setAttribute('aria-live', 'off'); keepStatus(); setInterval(keepStatus, 500); }, 1200);
    },
  });
  const retry = e => {
    if (A.kind !== 'webgpu') throw e;
    let why = '';
    try { why = self.amenuiGlue.webgpu.start_failure(); } catch (_) { /* it did not get that far */ }
    console.warn('the WebGPU-only module could not start (' + (why || 'error') + '); starting the full one', e);
    api = null;
    const c = stage.querySelector('.c');
    // A canvas keeps the first kind of context it gave: the full module asks for its own.
    c.replaceWith(c.cloneNode(false));
    A.load('full');
    return start();
  };
  start().catch(retry).then(a => {
    api = a;
    // The module is in this browser's cache now: a docs page's stage can start as it is scrolled to.
    try { localStorage.setItem('amenui.module', A.files[A.kind === 'webgpu' ? 'wasmGpu' : 'wasm']); } catch (e) {}
    // The search takes a canvas route inside the running app.
    A.go = p => { a.route(p === '' ? '/' : p); };
    a.locale(A.locale);
    if (location.hash.startsWith('#look:')) a.route(location.hash);
    routed(a.w.path(), a.w.hash());
    // The bar's links to canvas pages are taken inside the running app (so the page change plays);
    // with a modifier key or another button they open as links do.
    D.addEventListener('click', e => {
      const l = e.target.closest && e.target.closest('a[data-route]');
      // Links to the docs (HTML pages) navigate as links do.
      if (l && l.dataset.route.startsWith('docs/')) return;
      if (!l || l.dataset.skip || l.dataset.play || e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
      e.preventDefault();
      D.getElementById('sheet').classList.remove('open');
      // The navigation's front starts at the hand (the link's centre for a keyboard activation).
      const r = l.getBoundingClientRect(), c = D.querySelector('#stage .c'), k = c ? c.getBoundingClientRect() : { left: 0, top: 0 };
      const [x, y] = e.detail === 0 ? [r.left + r.width / 2, r.top + r.height / 2] : [e.clientX, e.clientY];
      if (a.w.pressed_at) a.w.pressed_at(x - k.left, y - k.top);
      a.route(l.dataset.route === '' ? '/' : l.dataset.route);
    });
    addEventListener('popstate', () => {
      const p = location.pathname.slice((A.base + A.prefix).length);
      a.route(p === '' ? '/' : p);
    });
  }).catch(e => A.fail(e));
}
site();
