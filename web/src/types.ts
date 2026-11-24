export type AiAlgorithm = 'paranoid' | 'maxN' | 'greedy';

export interface AiConfig {
  algorithm: AiAlgorithm;
  timeBudgetMs: number;
  maxDepth: number;
}

export type PlayerKind =
  | { type: 'human' }
  | ({ type: 'ai' } & AiConfig);

export interface PlayerConfig {
  name: string;
  symbol: string;
  color: string;
  kind: PlayerKind;
}

export interface GameConfig {
  boardSize: number;
  winLength: number;
  fourCorners: boolean;
  players: PlayerConfig[];
}

export type WinKind = 'row' | 'column' | 'diagonal' | 'fourCorners';

export type Outcome =
  | { status: 'ongoing' }
  | { status: 'win'; player: number; kind: WinKind; cells: number[] }
  | { status: 'draw' };

export interface MoveRecord {
  player: number;
  cell: number;
}

export interface GameSnapshot {
  boardSize: number;
  winLength: number;
  fourCorners: boolean;
  cells: (number | null | undefined)[];
  players: PlayerConfig[];
  currentTurn: number;
  outcome: Outcome;
  history: MoveRecord[];
}

export interface SearchTelemetry {
  algorithm: AiAlgorithm;
  depthReached: number;
  nodesEvaluated: number;
  ttHits: number;
  elapsedMs: number;
  nodesPerSec: number;
}

export interface SearchResult {
  bestMove: number;
  score: number;
  telemetry: SearchTelemetry;
}

export interface AiTurnResponse {
  result: SearchResult;
  state: GameSnapshot;
}
