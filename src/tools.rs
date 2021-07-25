use web_sys::CanvasRenderingContext2d;

/**
 * Draws a rounded rectangle using the current state of the canvas.
 * If you omit the last three params, it will draw a rectangle
 * outline with a 5 pixel border radius
 * @param {CanvasRenderingContext2D} ctx
 * @param {Number} x The top left x coordinate
 * @param {Number} y The top left y coordinate
 * @param {Number} width The width of the rectangle
 * @param {Number} height The height of the rectangle
 * @param {Number} [radius = 5] The corner radius; It can also be an object 
 *                 to specify different radii for corners
 * @param {Number} [radius.tl = 0] Top left
 * @param {Number} [radius.tr = 0] Top right
 * @param {Number} [radius.br = 0] Bottom right
 * @param {Number} [radius.bl = 0] Bottom left
 * @param {Boolean} [fill = false] Whether to fill the rectangle.
 * @param {Boolean} [stroke = true] Whether to stroke the rectangle.
 */
 pub fn draw_round_rect(ctx: &CanvasRenderingContext2d, x: f64, y: f64, width: f64, height: f64, radius: f64, fill: bool, stroke: bool) {
    let tl = radius;
    let tr = radius;
    let br = radius;
    let bl = radius;

    ctx.begin_path();
    ctx.move_to(x + tl, y);
    ctx.line_to(x + width - tr, y);
    ctx.quadratic_curve_to(x + width, y, x + width, y + tr);
    ctx.line_to(x + width, y + height - br);
    ctx.quadratic_curve_to(x + width, y + height, x + width - br, y + height);
    ctx.line_to(x + bl, y + height);
    ctx.quadratic_curve_to(x, y + height, x, y + height - bl);
    ctx.line_to(x, y + tl);
    ctx.quadratic_curve_to(x, y, x + tl, y);
    ctx.close_path();
    if fill {
      ctx.fill();
    }
    if stroke {
      ctx.stroke();
    }
  }