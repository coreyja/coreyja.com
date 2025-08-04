import React from 'react'

interface DateFilterProps {
  value: number
  onChange: (days: number) => void
}

export const DateFilter: React.FC<DateFilterProps> = ({ value, onChange }) => {
  const handleChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    onChange(parseInt(event.target.value, 10))
  }

  return (
    <select
      value={value}
      onChange={handleChange}
      style={{
        padding: '6px 12px',
        borderRadius: '4px',
        border: '1px solid #ccc',
        backgroundColor: 'white',
        fontSize: '14px',
        cursor: 'pointer',
      }}
    >
      <option value="1">1 day</option>
      <option value="3">3 days</option>
      <option value="7">7 days</option>
    </select>
  )
}
