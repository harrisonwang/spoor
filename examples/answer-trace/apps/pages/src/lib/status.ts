import type { Verdict } from '@answer-trace/protocol'

export interface StatusStyle {
  label: string
  short: string
  color: string
  bg: string
  border: string
  text: string
  wash: string
  soft: string
}

export const STATUS: Record<Verdict, StatusStyle> = {
  supported: {
    label: '已核验',
    short: '✓',
    color: '#15A34A',
    bg: '#E7F6ED',
    border: '#BFE8CD',
    text: '#0F7A38',
    wash: 'rgba(21, 163, 74, 0.18)',
    soft: 'rgba(21, 163, 74, 0.07)',
  },
  partial: {
    label: '需复核',
    short: '!',
    color: '#D97706',
    bg: '#FCF0DA',
    border: '#F1D19B',
    text: '#92400E',
    wash: 'rgba(217, 119, 6, 0.2)',
    soft: 'rgba(217, 119, 6, 0.08)',
  },
  unsupported: {
    label: '无法核验',
    short: '✗',
    color: '#DC2626',
    bg: '#FBE6E6',
    border: '#F3BABA',
    text: '#991B1B',
    wash: 'rgba(220, 38, 38, 0.18)',
    soft: 'rgba(220, 38, 38, 0.08)',
  },
}
