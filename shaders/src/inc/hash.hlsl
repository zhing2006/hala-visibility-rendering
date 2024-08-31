#ifndef _HASH_HLSL_
#define _HASH_HLSL_

uint hash_add(uint hash, uint elem) {
  elem *= 0xcc9e2d51;
  elem = (elem << 15) | (elem >> (32 - 15));
  elem *= 0x1b873593;

  hash ^= elem;
  hash = (hash << 13) | (hash >> (32 - 13));
  hash = hash * 5 + 0xe6546b64;
  return hash;
}

uint hash_mix(uint hash) {
  hash ^= hash >> 16;
  hash *= 0x85ebca6b;
  hash ^= hash >> 13;
  hash *= 0xc2b2ae35;
  hash ^= hash >> 16;
  return hash;
}

#endif // _HASH_HLSL_