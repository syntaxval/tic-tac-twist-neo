import init, { WasmGame } from '../wasm/tic_tac_twist_wasm';
import type { AiTurnResponse, GameConfig, GameSnapshot } from '../types';

type WorkerRequest =
  | { id: number; type: 'init'; config: GameConfig }
  | { id: number; type: 'playMove'; cell: number }
  | { id: number; type: 'playAiTurn' }
  | { id: number; type: 'undo' }
  | { id: number; type: 'reset' };

type WorkerResponse =
  | { id: number; ok: true; payload: GameSnapshot | AiTurnResponse }
  | { id: number; ok: false; error: string };

let wasmReady: Promise<unknown> | null = null;
let game: WasmGame | null = null;

async function ensureWasm(): Promise<void> {
  if (!wasmReady) {
    wasmReady = init();
  }
  await wasmReady;
}

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const req = event.data;
  try {
    await ensureWasm();

    let payload: GameSnapshot | AiTurnResponse;
    switch (req.type) {
      case 'init': {
        if (game) {
          game.free();
        }
        game = new WasmGame(req.config);
        payload = game.state() as GameSnapshot;
        break;
      }
      case 'playMove': {
        if (!game) throw new Error('Game not initialized');
        payload = game.play_move(req.cell) as GameSnapshot;
        break;
      }
      case 'playAiTurn': {
        if (!game) throw new Error('Game not initialized');
        payload = game.play_ai_turn() as AiTurnResponse;
        break;
      }
      case 'undo': {
        if (!game) throw new Error('Game not initialized');
        payload = game.undo() as GameSnapshot;
        break;
      }
      case 'reset': {
        if (!game) throw new Error('Game not initialized');
        payload = game.reset() as GameSnapshot;
        break;
      }
    }

    const res: WorkerResponse = { id: req.id, ok: true, payload };
    self.postMessage(res);
  } catch (err) {
    const res: WorkerResponse = {
      id: req.id,
      ok: false,
      error: err instanceof Error ? err.message : String(err),
    };
    self.postMessage(res);
  }
};
