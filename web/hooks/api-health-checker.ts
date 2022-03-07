/**
 * This hook exposes an rxjs observer that can be subscribed to that 
 * continually checks the API server's status check endpoint.
 * In a production app I would have this thing adjust it's interval
 * timer and increase the frequency when interesting things are 
 * happening, and decrease it when nothing is happening.
 * 
 * To do this I'd add another state value for frequency, and then
 * increase the interval timer to be 1 second, and do a timestamp
 * check during each fire to allow me to more precisely control the delay.
 */

import { useEffect, useState } from 'react'
import { interval } from 'rxjs'

const API_URI = process.env.NEXT_PUBLIC_API_URI

export function useApiHealthChecker() {
  const [isHealthy, setIsHealthy] = useState(false)
  const [lastResponse, setLastResponse] = useState(Date.now())

  // We need to track when the last response was to be able to detect the
  // API going down. When the API is stopped, CORS no longer succeeds, and
  // the promises never resolve or reject, so we will assume 2.5 intervals
  // without the promise resolving means that the API is unavailable.
  async function check() {
    if (Date.now() - lastResponse > 12500) {
      setIsHealthy(false)
    }
    const response = await apiCall()
    setLastResponse(Date.now())
    if (response !== isHealthy) {
      setIsHealthy(response)
    }
  }
  
  useEffect(() => {
    const s = interval(5000).subscribe(check)
    return () => s?.unsubscribe()
  }, [])

  return isHealthy
}

async function apiCall(): Promise<boolean> {
  try {
    const res = await fetch(`${API_URI}/status`)
    const text = await res.text()
    return text === "OK"
  } catch (_e) {
    return false
  }
}
