// Phase 2: expand this module with GameState, score, wave, camera, and audio queue.
// Game-wide state lives here (not in the ECS world) because hecs is optimised for
// many same-shaped entities, not singleton resources accessed by every system.
