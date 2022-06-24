import Image from 'next/image'

export function Logo() {
  return (
    <div className="flex items-center gap-2">
      <Image src="/logo.png" alt="Logo" width={30} height={30} />
      <span className="text-xl font-semibold">Anchor</span>
    </div>
  )
}
