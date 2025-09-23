import { getMatches } from '@tauri-apps/plugin-cli'
import { useEffect, useState } from 'react'

const useCli = () => {
  const [source, setSource] = useState<string | null>(null)

  useEffect(() => {
    const fetchMatches = async () => {
      const data = await getMatches()
      if (data.args['source'].value) {
        setSource(data.args['source'].value as string)
      }
    }

    fetchMatches()
  }, [])

  return source
}

export default useCli