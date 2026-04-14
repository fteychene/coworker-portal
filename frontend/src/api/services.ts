import { z } from 'zod'
import { apiFetch } from './client'

const VoucherSpecSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('Monthly') }),
  z.object({ kind: z.literal('Book'), amount: z.number(), duration: z.number() }),
])

export const ServiceSchema = z.object({
  id: z.number(),
  name: z.string(),
  description: z.string(),
  price: z.number(),
  voucher_spec: VoucherSpecSchema,
  external_service_id: z.number(),
})

export type Service = z.infer<typeof ServiceSchema>
export type VoucherSpec = z.infer<typeof VoucherSpecSchema>

const ServicesResponseSchema = z.object({
  data: z.array(ServiceSchema),
})

export async function listServices(): Promise<Service[]> {
  const raw = await apiFetch<unknown>('/api/services')
  return ServicesResponseSchema.parse(raw).data
}
