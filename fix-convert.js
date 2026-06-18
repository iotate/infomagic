import fs from 'fs';

let pages = fs.readFileSync('d:/Dev/Apps/ais/apps/infoimage/infomagic/src/views/Step3_Pages.vue', 'utf8');

// Fix getImageUrl - for absolute paths, convertFileSrc should work directly
// The issue is that convertFileSrc might need the protocol parameter in v2
pages = pages.replace(
  /function getImageUrl\(imagePath: string \| undefined\): string \{[\s\S]*?return convertFileSrc\(normalizedPath\);[\s\S]*?\}/,
  `function getImageUrl(imagePath: string | undefined): string {
    if (!imagePath) return '';
    // Normalize Windows backslashes to forward slashes
    const normalizedPath = imagePath.replace(/\\\\/g, '/');
    // For Tauri v2, convertFileSrc handles absolute paths directly
    // The 'asset' protocol is the default
    return convertFileSrc(normalizedPath, 'asset');
  }`
);

fs.writeFileSync('d:/Dev/Apps/ais/apps/infoimage/infomagic/src/views/Step3_Pages.vue', pages, 'utf8');
console.log('Updated getImageUrl with protocol parameter');

// Same fix for TemplateManagement.vue
let template = fs.readFileSync('d:/Dev/Apps/ais/apps/infoimage/infomagic/src/views/TemplateManagement.vue', 'utf8');

template = template.replace(
  /function getPreviewUrl[\s\S]*?return convertFileSrc\(normalizedPath\)/,
  `function getPreviewUrl(template: TemplateInfo, type: 'front' | 'content' | 'back'): string | undefined {
  let path: string | undefined
  if (type === 'front') path = template.front_cover_path
  else if (type === 'content') path = template.content_path
  else path = template.back_cover_path

  if (!path) return undefined

  // Normalize Windows backslashes to forward slashes
  const normalizedPath = path.replace(/\\\\/g, '/');
  return convertFileSrc(normalizedPath, 'asset')`
);

fs.writeFileSync('d:/Dev/Apps/ais/apps/infoimage/infomagic/src/views/TemplateManagement.vue', template, 'utf8');
console.log('Updated getPreviewUrl with protocol parameter');

console.log('Done');
