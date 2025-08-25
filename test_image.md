# Image Display Testing

## Fixed Issues:
1. **Size Limits**: Images larger than 5MB are rejected for display
2. **Dimension Limits**: Images larger than 4000x4000 pixels are rejected
3. **Panic Protection**: Image processing is wrapped in catch_unwind to prevent crashes
4. **Fallback**: If image display fails, it falls back to text display
5. **Toggle**: Ctrl+I to enable/disable image display
6. **Auto-disable**: If image processing fails, it automatically disables for the session

## Test URLs:
- Small image: `https://httpbin.org/image/png`
- Medium image: `https://picsum.photos/800/600`
- Large image: `https://images.unsplash.com/photo-1575936123452-b67c3203c357?q=80&w=1470`

## Controls:
- **Ctrl+I**: Toggle image display on/off
- **Ctrl+C**: Quit application
- **Tab**: Switch between response tabs (Response/Headers/Cookies)

## Behavior:
- Images are automatically detected by Content-Type header
- Small images display immediately
- Large images show size warning
- Failed images fall back to text display
- UI remains responsive during image processing