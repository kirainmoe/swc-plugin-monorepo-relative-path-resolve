export { foo } from '@/foo';
export { getBar } from '@/bar/bar';

export const asyncLoader = async () => {
  const result = await import('@/bar/bar');
  result.getBar().then(result => console.log(result));
};