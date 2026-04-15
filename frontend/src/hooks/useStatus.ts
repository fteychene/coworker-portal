import { useEffect, useState } from 'react'

interface AppStatus {
  invoice_available: boolean
}

// Module-level cache so multiple components share one fetch per page load.
let cached: AppStatus | null = null
let promise: Promise<AppStatus> | null = null

async function fetchStatus(): Promise<AppStatus> {
  if (cached) return cached
  if (!promise) {
    promise = fetch('/api/status')
      .then(r => r.json() as Promise<AppStatus>)
      .then(s => { cached = s; return s })
      .catch(() => { promise = null; return { invoice_available: false } })
  }
  return promise
}

export function useStatus() {
  const [status, setStatus] = useState<AppStatus>(cached ?? { invoice_available: false })

  useEffect(() => {
    fetchStatus().then(setStatus)
  }, [])

  return status
}
