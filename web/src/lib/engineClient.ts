import type { AiTurnResponse, GameConfig, GameSnapshot } from '../types';

export class EngineClient {
  private worker: Worker;
  private nextId = 1;
  private pending = new Map<
    number,
    { resolve: (value: unknown) => void; reject: (err: Error) => void }
  >();

  constructor() {
    this.worker = new Worker(
      new URL('../worker/engine.worker.ts', import.meta.url),
      { type: 'module' }
    );

    this.worker.onmessage = (event: MessageEvent) => {
      const { id, ok, payload, error } = event.data;
      const entry = this.pending.get(id);
      if (!entry) return;
      this.pending.delete(id);
      if (ok) {
        entry.resolve(payload);
      } else {
        entry.reject(new Error(error));
      }
    };
  }

  private call<T>(message: Record<string, unknown>): Promise<T> {
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, {
        resolve: resolve as (value: unknown) => void,
        reject,
      });
      this.worker.postMessage({ id, ...message });
    });
  }

  init(config: GameConfig): Promise<GameSnapshot> {
    // Strip any framework reactive Proxies before structured-cloning to the Worker.
    const plainConfig: GameConfig = JSON.parse(JSON.stringify(config));
    return this.call<GameSnapshot>({ type: 'init', config: plainConfig });
  }

  playMove(cell: number): Promise<GameSnapshot> {
    return this.call<GameSnapshot>({ type: 'playMove', cell });
  }

  playAiTurn(): Promise<AiTurnResponse> {
    return this.call<AiTurnResponse>({ type: 'playAiTurn' });
  }

  undo(): Promise<GameSnapshot> {
    return this.call<GameSnapshot>({ type: 'undo' });
  }

  reset(): Promise<GameSnapshot> {
    return this.call<GameSnapshot>({ type: 'reset' });
  }

  destroy(): void {
    this.worker.terminate();
    this.pending.clear();
  }
}
