export interface Task {
  id: string
  text: string
  completed: boolean
  createdAt: number
  /** Hex color for the accent strip, generated from task text */
  color: string
  /** Optional deadline timestamp */
  ddl?: number
}
