#!/usr/bin/env swift
// Renders the Stynx app icon (terminal chevron + cursor on a violet gradient)
// into an .iconset directory. Pipe through iconutil to get AppIcon.icns:
//
//   swift stynx-desktop/scripts/generate-icon.swift /tmp/AppIcon.iconset
//   iconutil -c icns /tmp/AppIcon.iconset -o stynx-desktop/Resources/AppIcon.icns

import AppKit

let outDir = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "AppIcon.iconset"
try FileManager.default.createDirectory(atPath: outDir, withIntermediateDirectories: true)

func render(_ pixels: Int) -> NSBitmapImageRep {
    let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil, pixelsWide: pixels, pixelsHigh: pixels,
        bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
        colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)

    let size = CGFloat(pixels)
    let s = size / 1024.0

    // macOS icon grid: 824pt rounded square centered on a 1024pt canvas.
    let inset = 100.0 * s
    let square = NSRect(x: inset, y: inset, width: size - 2 * inset, height: size - 2 * inset)
    let radius = 185.0 * s
    let squircle = NSBezierPath(roundedRect: square, xRadius: radius, yRadius: radius)

    // Drop shadow behind the tile.
    let shadow = NSShadow()
    shadow.shadowColor = NSColor.black.withAlphaComponent(0.35)
    shadow.shadowBlurRadius = 24 * s
    shadow.shadowOffset = NSSize(width: 0, height: -10 * s)
    NSGraphicsContext.current?.saveGraphicsState()
    shadow.set()
    NSColor(calibratedRed: 0.13, green: 0.09, blue: 0.32, alpha: 1).setFill()
    squircle.fill()
    NSGraphicsContext.current?.restoreGraphicsState()

    // Violet → deep indigo diagonal gradient.
    NSGradient(colors: [
        NSColor(calibratedRed: 0.51, green: 0.32, blue: 0.98, alpha: 1),  // #8252FA
        NSColor(calibratedRed: 0.29, green: 0.15, blue: 0.72, alpha: 1),  // #4A26B8
        NSColor(calibratedRed: 0.10, green: 0.06, blue: 0.30, alpha: 1),  // #1A0F4D
    ], atLocations: [0.0, 0.55, 1.0], colorSpace: .deviceRGB)!
        .draw(in: squircle, angle: -65)

    // Soft radial highlight top-left, clipped to the tile (liquid-glass sheen).
    NSGraphicsContext.current?.saveGraphicsState()
    squircle.addClip()
    NSGradient(starting: NSColor.white.withAlphaComponent(0.16), ending: .clear)?
        .draw(fromCenter: NSPoint(x: size * 0.32, y: size * 0.88), radius: 0,
              toCenter: NSPoint(x: size * 0.32, y: size * 0.88), radius: 620 * s,
              options: [])
    // Hairline inner border for definition.
    NSColor.white.withAlphaComponent(0.16).setStroke()
    let border = NSBezierPath(roundedRect: square.insetBy(dx: 3 * s, dy: 3 * s),
                              xRadius: radius - 3 * s, yRadius: radius - 3 * s)
    border.lineWidth = 6 * s
    border.stroke()

    // Glyph: prompt chevron ❯ plus a block cursor, drawn as paths.
    let glyph = NSBezierPath()
    let stroke = 96.0 * s
    // Chevron: from (330,690) to (560,512) to (330,334) in 1024-space.
    glyph.move(to: NSPoint(x: 330 * s, y: 700 * s))
    glyph.line(to: NSPoint(x: 560 * s, y: 512 * s))
    glyph.line(to: NSPoint(x: 330 * s, y: 324 * s))
    glyph.lineWidth = stroke
    glyph.lineCapStyle = .round
    glyph.lineJoinStyle = .round

    let glow = NSShadow()
    glow.shadowColor = NSColor(calibratedRed: 0.75, green: 0.60, blue: 1.0, alpha: 0.9)
    glow.shadowBlurRadius = 34 * s
    glow.shadowOffset = .zero
    NSGraphicsContext.current?.saveGraphicsState()
    glow.set()
    NSColor.white.setStroke()
    glyph.stroke()

    // Cursor bar to the right of the chevron, baseline-aligned.
    let bar = NSBezierPath(roundedRect: NSRect(x: 620 * s, y: 324 * s - stroke / 2,
                                               width: 230 * s, height: stroke),
                           xRadius: stroke / 2, yRadius: stroke / 2)
    NSColor.white.setFill()
    bar.fill()
    NSGraphicsContext.current?.restoreGraphicsState()

    NSGraphicsContext.current?.restoreGraphicsState()
    NSGraphicsContext.restoreGraphicsState()
    return rep
}

let variants: [(Int, String)] = [
    (16, "icon_16x16"), (32, "icon_16x16@2x"),
    (32, "icon_32x32"), (64, "icon_32x32@2x"),
    (128, "icon_128x128"), (256, "icon_128x128@2x"),
    (256, "icon_256x256"), (512, "icon_256x256@2x"),
    (512, "icon_512x512"), (1024, "icon_512x512@2x"),
]
for (px, name) in variants {
    let rep = render(px)
    try rep.representation(using: .png, properties: [:])!
        .write(to: URL(fileURLWithPath: "\(outDir)/\(name).png"))
}
print("wrote \(variants.count) PNGs to \(outDir)")
