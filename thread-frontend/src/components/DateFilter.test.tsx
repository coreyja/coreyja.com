import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { DateFilter } from './DateFilter'

describe('DateFilter', () => {
  it('renders with default selection of 3 days', () => {
    const onChange = vi.fn()
    render(<DateFilter value={3} onChange={onChange} />)

    const select = screen.getByRole('combobox')
    expect(select).toHaveValue('3')
  })

  it('displays all available options', async () => {
    const onChange = vi.fn()
    render(<DateFilter value={3} onChange={onChange} />)

    const select = screen.getByRole('combobox')
    const user = userEvent.setup()
    await user.click(select)

    expect(screen.getByRole('option', { name: '1 day' })).toBeInTheDocument()
    expect(screen.getByRole('option', { name: '3 days' })).toBeInTheDocument()
    expect(screen.getByRole('option', { name: '7 days' })).toBeInTheDocument()
  })

  it('calls onChange when selection changes', async () => {
    const onChange = vi.fn()
    render(<DateFilter value={3} onChange={onChange} />)

    const select = screen.getByRole('combobox')
    const user = userEvent.setup()

    await user.selectOptions(select, '7')

    expect(onChange).toHaveBeenCalledWith(7)
  })
})
