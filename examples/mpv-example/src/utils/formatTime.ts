const formatTime = (seconds: number | null | undefined): string => {
  if (seconds === null || seconds === undefined || isNaN(seconds)) {
    return '00:00'
  }
  const flooredSeconds = Math.floor(seconds)
  const m = Math.floor(flooredSeconds / 60)
  const s = flooredSeconds % 60
  return `${m < 10 ? '0' : ''}${m}:${s < 10 ? '0' : ''}${s}`
}

export default formatTime