#version 450

layout(location = 0) in vec2 screen_coord;
layout(location = 0) out vec4 colour;

layout(set = 0, binding = 0) uniform sampler2D map;
layout(set = 0, binding = 1) uniform sampler2D palette;


layout(set = 0, binding = 2) uniform Globals {
  double time;
  uvec2 screen_size;
  float fps;
} global;


const uint cycle_none = 0;
const uint cycle_rotate_1 = 1;
const uint cycle_rotate_2 = 2;
const uint cycle_triange = 3;

struct PaletteCycle {
  float start;
  float end;
};

layout(set = 0, binding = 3) uniform PaletteCycles {
  float speed;
  uint style;
  PaletteCycle cycles[2];
} pc;


struct Translation {
  float duration;
  vec2 velocity;
  vec2 acceleration;
};

layout(set = 0, binding = 4) uniform Translations {
  Translation translations[4];
} translations;


const uint dist_none = 0;
const uint dist_horizontal = 1;
const uint dist_interlaced = 2;
const uint dist_vertical = 3;
const uint dist_shear = 4;

struct Distortion {
  float duration;
  uint style;
  float frequency;
  float frequency_delta;
  float amplitude;
  float amplitude_delta;
  float compression;
  float compression_delta;
  float speed;
};

layout(set = 0, binding = 5) uniform Distortions {
  Distortion distortions[4];
} distortions;

vec2 translation(in vec2 v, in vec2 a, in float t) {
  // TODO why is acceleration too low?
  return v * t + a * t * t * (global.fps / 2.);
}

vec2 distortion(in Distortion d, in vec2 screen, in vec2 screen_coord, in float t) {
  t *= float(global.fps / 2.);
  vec2 offset = vec2(0);
  //float dist = (d.amplitude + d.amplitude_delta * t) * sin(screen.y * (d.frequency + d.frequency_delta * t) + d.speed * t);
  float ampl = (d.amplitude + d.amplitude_delta * t);
  float freq = (d.frequency + d.frequency_delta * t);
  float speed = d.speed * t;
  float dist = ampl * sin(screen.y * freq + speed);
  float comp = d.compression + d.compression_delta * t * 2.;
  switch (d.style) {
    case dist_none:
      break;
    case dist_horizontal:
      offset.x = dist;
      break;
    case dist_interlaced:
      offset.x = mod(screen.y, 2.) > 1. ? dist : -dist;
      break;
    case dist_vertical:
      offset.y = dist;
      offset.y += screen.y * (comp / screen_coord.y);
      break;
    case dist_shear:
      offset.x = mod(screen.y, 2.) > 1. ? dist : -dist;
      offset.x += screen.y * (comp / screen_coord.y);
      break;
  }
  return offset;
}

void main() {
    double frame = global.time * double(global.fps);
    vec2 screen = global.screen_size * screen_coord;
    
    // TODO Create translation code
    // TODO Duration 0 is constant velocity
    vec2 scroll = vec2(0);
    /*{
      double total = 0;
      // Calculate total duration
      for (int i = 0; i < 4; i++) total += translations.translations[i].duration;
      // Calculate number of full cycles that have passed
      int loop = int(global.time / total);
      // Calculate point in current cycle
      double rem = mod(global.time, total);
      // Skip to current cycle
      for (int i = 0; i < 4; i++) {
        Translation t = translations.translations[i];
        if (t.duration > 0) {
          float dt = t.duration * loop;
          scroll += translation(t.velocity, t.acceleration, dt);
        }
      }
      // Move to point in current cycle
      for (int i = 0; i < 4; i++) {
        Translation t = translations.translations[i];
        if (t.duration == 0) {
          float dt = float(global.time / double(global.fps));
          scroll += translation(t.velocity, t.acceleration, dt);
        }
        // Skip passed translation
        else if (t.duration < rem) {
          float dt = t.duration;
          scroll += translation(t.velocity, t.acceleration, dt);
          rem -= t.duration;
        }
        // Partial translation
        else {
          float dt = float(rem / 2.);
          scroll += translation(t.velocity, t.acceleration, dt);
          break;
        }
      }
    }*/
    {
      double total = 0;
      for (int i = 0; i < 4; i++) total += translations.translations[i].duration;
      int loop = int(global.time / total);
      double rem = mod(global.time, total);
      /*for (int i = 0; i < 4; i++) {
        Translation t = translations.translations[i];
        if (t.duration > 0) {
          float dt = t.duration * float(loop);
          scroll += translation(t.velocity, t.acceleration, dt);
        }
      }*/
      for (int i = 0; i < 4; i++) {
        Translation t = translations.translations[i];
        if (t.duration == 0) {
          float dt = float(frame);//float(global.time / double(global.fps));
          scroll += translation(t.velocity, t.acceleration, dt);
        }
        else if (t.duration < rem) {
          float dt = t.duration;
          scroll += translation(t.velocity, t.acceleration, dt);
          rem -= t.duration;
        }
        else {
          float dt = float(rem);
          scroll += translation(t.velocity, t.acceleration, dt);
          break;
        }
      }
    }
    
    scroll /= global.screen_size;
    
    // TODO: Abide by distortion duration
    vec2 offset = vec2(0);
    {
      double total = 0;
      for (int i = 0; i < 4; i++) total += distortions.distortions[i].duration;
      double rem = mod(global.time, total);
      for (int i = 0; i < 4; i++) {
        Distortion d = distortions.distortions[i];
        if (d.duration == 0) {
          float dt = float(global.time);
          offset += distortion(d, screen, global.screen_size, dt);
        }
        else if (d.duration < rem) {
          rem -= d.duration;
        }
        else {
          float dt = float(rem);
          // TODO: What is this?
          offset += distortion(d, screen, global.screen_size, dt);
          break;
        }
      }
    }
    offset /= global.screen_size;
    
    vec2 p = screen_coord;
    // TODO: Rename these
    p += offset;
    p += scroll;
//    p /= global.screen_size;

    float c = texture(map, p).r;
    if (c == 0) discard; // 0 is transparent
    // Map 256 values to 16, 256 / 16 = 16
    c *= 16.;

    
    float i = c;
    {
      float ranges[2] = {0, 0};
      for (int r = 0; r < 2; r++) ranges[r] = (pc.cycles[r].end - pc.cycles[r].start) + (1./16.);
      int cycle = -1;
      // Check whether colour is in a cycle
      if (c >= pc.cycles[0].start && c <= pc.cycles[0].end + (1./16.))
        cycle = 0;
      else if (c >= pc.cycles[1].start && c <= pc.cycles[1].end + (1./16.))
        cycle = 1;

      // TODO: Framerate is half frame rate?
      float t = float((frame / 2.) / double(pc.speed));

      if (cycle >= 0) {
        switch (pc.style) {
          case cycle_rotate_1:
          case cycle_rotate_2:
            i = pc.cycles[cycle].start + mod(c + t, ranges[cycle]);
            break;
          case cycle_triange:
            i = pc.cycles[cycle].start + abs(mod(c + t, ranges[cycle] * 2.) - ranges[cycle]);
            break;
        }
      }
    }

    colour.rgb = vec3(c); // Just map
    colour.rgb = vec3(i); // Cycling map
    //colour.rgb = texture(palette, screen_coord).rgb; // Just palette
    //colour.rgb = texture(palette, vec2(c, 0.0)).rgb; // Colourised map (initial state)

    colour.rgb = texture(palette, vec2(i, 0.0)).rgb;
    colour.a = 1.0;
}
