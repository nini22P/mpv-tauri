import { useState, useRef, useCallback, useEffect } from 'react'

const useAutoHide = (duration: number, initialVisible = false) => {
  const [visible, setVisible] = useState(initialVisible)
  const timeoutRef = useRef<number | null>(null)

  const show = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current)
    }
    setVisible(true)
    timeoutRef.current = window.setTimeout(() => {
      setVisible(false)
    }, duration)
  }, [duration])

  const hide = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current)
    }
    setVisible(false)
  }, [])

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
      }
    }
  }, [])

  return { visible, show, hide }
}

export default useAutoHide